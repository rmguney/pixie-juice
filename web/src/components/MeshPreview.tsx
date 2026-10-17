// 3D preview with plain three.js. One renderer lives as long as the preview;
// frames are drawn only when something changes (load, resize, orbiting) or an
// animation plays, so an idle preview costs nothing.

import { useEffect, useMemo, useRef, useState } from 'react';
import {
  AnimationMixer,
  type Camera,
  Clock,
  DirectionalLight,
  GridHelper,
  Group,
  HemisphereLight,
  type LineBasicMaterial,
  type Mesh,
  type MeshStandardMaterial,
  type Object3D,
  PerspectiveCamera,
  type PointLight,
  Quaternion,
  Scene,
  type SpotLight,
  Vector3,
  WebGLRenderer,
} from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import type { MeshFormat } from '../core/types';
import { companionCandidates } from '../utils/companions';
import { disposeModel, loadModel } from '../utils/meshLoaders';

interface View {
  scene: Scene;
  camera: PerspectiveCamera;
  controls: OrbitControls;
  grid: GridHelper;
  sun: DirectionalLight;
  sky: HemisphereLight;
  /** The colour for meshes that bring none of their own. */
  surface: string;
  /** Plays the model's animation while set. */
  mixer: AnimationMixer | null;
  playing: boolean;
  redraw: () => void;
}

const CAMERA_DIRECTION = new Vector3(1, 0.75, 1).normalize();
const SKY_INTENSITY = 2.2;
const reducedMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

type Status = 'loading' | { error: string } | { animated: boolean };

export default function MeshPreview({ file, type, files }: { file: File; type: MeshFormat; files: File[] }) {
  const mount = useRef<HTMLDivElement>(null);
  const view = useRef<View | null>(null);
  const [status, setStatus] = useState<Status>('loading');
  const [playing, setPlaying] = useState(false);

  // A .gltf reloads when a file it may refer to is added or removed, not for any change to the list.
  const candidates = useMemo(() => companionCandidates(file, files), [files, file]);
  const candidateKey = candidates.map((f) => `${f.name}:${f.size}:${f.lastModified}`).join('|');
  const latestCandidates = useRef(candidates);
  useEffect(() => {
    latestCandidates.current = candidates; // before the load below, which reads it
  });

  useEffect(() => {
    const host = mount.current!;
    let renderer: WebGLRenderer;
    try {
      renderer = new WebGLRenderer({ antialias: true, alpha: true });
    } catch {
      setStatus({ error: 'WebGL is not available in this browser' });
      return;
    }
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    host.appendChild(renderer.domElement);

    const scene = new Scene();
    const sky = new HemisphereLight(0xffffff, 0x50505a, SKY_INTENSITY);
    scene.add(sky);
    const sun = new DirectionalLight(0xffffff, 2.2);
    sun.position.set(3, 5, 4);
    scene.add(sun);
    const token = (name: string) => getComputedStyle(host).getPropertyValue(name).trim();
    const makeGrid = () => new GridHelper(6, 24, token('--grid'), token('--grid'));
    let grid = makeGrid();
    scene.add(grid);

    const camera = new PerspectiveCamera(38, 1, 0.01, 200);
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.minDistance = 0.4;
    controls.maxDistance = 30;

    const clock = new Clock();
    let frame = 0;
    const draw = () => {
      frame = 0;
      const current = view.current;
      const delta = clock.getDelta();
      if (current?.mixer && current.playing) {
        current.mixer.update(delta);
        redraw(); // the next frame of the animation
      }
      controls.update(); // while damping, this emits "change" and asks for the next frame
      renderer.render(scene, camera);
    };
    const redraw = () => {
      frame ||= requestAnimationFrame(draw);
    };
    controls.addEventListener('change', redraw);

    const resize = () => {
      const { clientWidth: width, clientHeight: height } = host;
      if (!width || !height) return;
      renderer.setSize(width, height, false);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
      redraw();
    };
    const observer = new ResizeObserver(resize);
    observer.observe(host);
    resize();

    view.current = { scene, camera, controls, grid, sun, sky, surface: token('--mesh'), mixer: null, playing: false, redraw };

    // The grid and the default mesh colour come from the theme: follow a change of colour scheme.
    const scheme = matchMedia('(prefers-color-scheme: dark)');
    const restyle = () => {
      const current = view.current;
      if (!current) return;
      const next = makeGrid();
      next.position.copy(grid.position);
      scene.remove(grid);
      grid.geometry.dispose();
      (grid.material as LineBasicMaterial).dispose();
      scene.add(next);
      grid = current.grid = next;
      current.surface = token('--mesh');
      scene.traverse((node) => {
        if (node.userData.surface) ((node as Mesh).material as MeshStandardMaterial).color.set(current.surface);
      });
      redraw();
    };
    scheme.addEventListener('change', restyle);

    return () => {
      view.current = null;
      scheme.removeEventListener('change', restyle);
      cancelAnimationFrame(frame);
      observer.disconnect();
      controls.dispose();
      grid.geometry.dispose();
      (grid.material as LineBasicMaterial).dispose();
      renderer.dispose();
      // Browsers cap live WebGL contexts; give this one back now rather than at garbage collection.
      renderer.forceContextLoss();
      renderer.domElement.remove();
    };
  }, []);

  useEffect(() => {
    const current = view.current;
    if (!current) return; // WebGL failed to start
    setStatus('loading');
    let cancelled = false;
    const group = new Group();
    file
      .arrayBuffer()
      .then((buffer) => loadModel(buffer, type, current.surface, latestCandidates.current))
      .then(({ object, box, animations }) => {
        if (cancelled) {
          disposeModel(object);
          return;
        }
        // Fit the model in a 2-unit box at the origin, standing on the grid.
        const scale = 2 / Math.max(box.getSize(new Vector3()).length(), 1e-6);
        const center = box.getCenter(new Vector3());
        group.add(object);
        group.scale.setScalar(scale);
        group.position.copy(center).multiplyScalar(-scale);
        current.grid.position.y = (box.min.y - center.y) * scale;
        current.scene.add(group);
        group.updateMatrixWorld(true);

        // The file's own lights replace the default sun (the sky stays as fill). The model was
        // scaled to fit, so lights that fade with distance are scaled to keep their brightness.
        const lights: Object3D[] = [];
        object.traverse((node) => (node as unknown as { isLight?: boolean }).isLight && lights.push(node));
        for (const light of lights) {
          const fading = light as PointLight | SpotLight;
          if ('distance' in fading && 'decay' in fading) {
            fading.intensity *= scale ** fading.decay;
            fading.distance *= scale;
          }
        }
        current.sun.visible = lights.length === 0;
        current.sky.intensity = lights.length === 0 ? SKY_INTENSITY : SKY_INTENSITY / 3;

        // The view starts where the file's first camera looks from, when the model is in
        // that camera's view (the model now sits at the origin); else from the front corner.
        let fileCamera: Camera | undefined;
        object.traverse((node) => {
          if (!fileCamera && (node as Camera).isCamera) fileCamera = node as Camera;
        });
        const fov = (fileCamera as PerspectiveCamera | undefined)?.isPerspectiveCamera ? (fileCamera as PerspectiveCamera).fov : 38;
        const position = fileCamera?.getWorldPosition(new Vector3()) ?? new Vector3();
        const forward = new Vector3(0, 0, -1).applyQuaternion(fileCamera?.getWorldQuaternion(new Quaternion()) ?? new Quaternion());
        const distance = -position.dot(forward); // to the model's centre, along the view
        const offAxis = position.clone().addScaledVector(forward, distance).length();
        const inView = fileCamera && distance > 0.5 && offAxis < distance * Math.tan((fov / 2) * (Math.PI / 180));
        current.controls.target.set(0, 0, 0);
        if (inView) {
          current.camera.position.copy(position);
          current.controls.target.copy(position).addScaledVector(forward, distance);
          current.camera.fov = fov;
        } else {
          current.camera.position.copy(CAMERA_DIRECTION).multiplyScalar(3.4);
          current.camera.fov = 38;
        }
        current.camera.updateProjectionMatrix();

        // Animation: the first clip loops, unless the viewer asked for less motion.
        if (animations.length > 0) {
          current.mixer = new AnimationMixer(object);
          current.mixer.clipAction(animations[0]).play();
          current.playing = !reducedMotion();
        }
        setPlaying(current.playing);
        current.redraw();
        setStatus({ animated: animations.length > 0 });
      })
      .catch((err: unknown) => {
        if (!cancelled) setStatus({ error: err instanceof Error ? err.message : String(err) });
      });
    return () => {
      cancelled = true;
      current.mixer?.stopAllAction();
      current.mixer = null;
      current.playing = false;
      current.sun.visible = true;
      current.sky.intensity = SKY_INTENSITY;
      current.scene.remove(group);
      disposeModel(group);
      // On unmount the renderer is already gone (effects clean up in order); only redraw a live view.
      if (view.current === current) current.redraw();
    };
    // candidateKey stands for the candidates (read through latestCandidates).
  }, [file, type, candidateKey]);

  const toggle = () => {
    const current = view.current;
    if (!current?.mixer) return;
    current.playing = !current.playing;
    setPlaying(current.playing);
    current.redraw();
  };

  const ready = typeof status === 'object' && 'animated' in status;
  const failed = typeof status === 'object' && 'error' in status;
  // The canvas host stays mounted in every state: the renderer outlives a failed model.
  return (
    <div className="mesh" data-testid="mesh-preview" aria-busy={status === 'loading'}>
      <div className="mesh-canvas" ref={mount} hidden={failed} />
      {status === 'loading' && (
        <div className="mesh-overlay">
          <div className="spinner" />
        </div>
      )}
      {ready && (
        <p className="mesh-hint">
          {status.animated && (
            <button type="button" className="text-button" onClick={toggle}>
              {playing ? 'Pause animation' : 'Play animation'}
            </button>
          )}
          Drag to orbit · scroll to zoom · right-drag to pan
        </p>
      )}
      {failed && (
        <div className="mesh-overlay">
          <p className="stage-message error-text">
            Cannot preview this {type.toUpperCase()}: {status.error}
          </p>
        </div>
      )}
    </div>
  );
}
