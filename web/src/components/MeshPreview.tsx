// 3D preview with plain three.js. One renderer lives as long as the preview;
// frames are drawn only when something changes (load, resize, orbiting), so
// an idle preview costs nothing.

import { useEffect, useRef, useState } from 'react';
import {
  DirectionalLight,
  GridHelper,
  Group,
  HemisphereLight,
  type LineBasicMaterial,
  PerspectiveCamera,
  Scene,
  Vector3,
  WebGLRenderer,
} from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import type { MeshFormat } from '../core/types';
import { disposeModel, loadModel } from '../utils/meshLoaders';

interface View {
  scene: Scene;
  camera: PerspectiveCamera;
  controls: OrbitControls;
  grid: GridHelper;
  /** The colour for meshes that bring none of their own. */
  surface: string;
  redraw: () => void;
}

const CAMERA_DIRECTION = new Vector3(1, 0.75, 1).normalize();

export default function MeshPreview({ file, type }: { file: File; type: MeshFormat }) {
  const mount = useRef<HTMLDivElement>(null);
  const view = useRef<View | null>(null);
  const [status, setStatus] = useState<'loading' | 'ready' | { error: string }>('loading');

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

    const style = getComputedStyle(host);
    const scene = new Scene();
    scene.add(new HemisphereLight(0xffffff, 0x50505a, 2.2));
    const sun = new DirectionalLight(0xffffff, 2.2);
    sun.position.set(3, 5, 4);
    scene.add(sun);
    const gridColor = style.getPropertyValue('--grid').trim();
    const grid = new GridHelper(6, 24, gridColor, gridColor);
    scene.add(grid);

    const camera = new PerspectiveCamera(38, 1, 0.01, 200);
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.minDistance = 0.4;
    controls.maxDistance = 30;

    let frame = 0;
    const draw = () => {
      frame = 0;
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

    view.current = { scene, camera, controls, grid, surface: style.getPropertyValue('--mesh').trim(), redraw };
    return () => {
      view.current = null;
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
      .then((buffer) => loadModel(new Uint8Array(buffer), type, current.surface))
      .then(({ object, box }) => {
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

        current.controls.target.set(0, 0, 0);
        current.camera.position.copy(CAMERA_DIRECTION).multiplyScalar(3.4);
        current.redraw();
        setStatus('ready');
      })
      .catch((err: unknown) => {
        if (!cancelled) setStatus({ error: err instanceof Error ? err.message : String(err) });
      });
    return () => {
      cancelled = true;
      current.scene.remove(group);
      disposeModel(group);
      // On unmount the renderer is already gone (effects clean up in order); only redraw a live view.
      if (view.current === current) current.redraw();
    };
  }, [file, type]);

  // The canvas host stays mounted in every state: the renderer outlives a failed model.
  return (
    <div className="mesh" data-testid="mesh-preview" aria-busy={status === 'loading'}>
      <div className="mesh-canvas" ref={mount} hidden={typeof status === 'object'} />
      {status === 'loading' && (
        <div className="mesh-overlay">
          <div className="spinner" />
        </div>
      )}
      {status === 'ready' && <p className="mesh-hint">Drag to orbit · scroll to zoom · right-drag to pan</p>}
      {typeof status === 'object' && (
        <div className="mesh-overlay">
          <p className="stage-message error-text">
            Cannot preview this {type.toUpperCase()}: {status.error}
          </p>
        </div>
      )}
    </div>
  );
}
