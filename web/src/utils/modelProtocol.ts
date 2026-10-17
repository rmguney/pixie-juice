// Messages between the page and the model worker (modelWorker.ts).

import type { SceneData } from './sceneTransfer';

export interface ModelRequest {
  id: number;
  type: 'gltf' | 'glb' | 'fbx';
  data: ArrayBuffer;
  /** For a .gltf: the files it may refer to (found by name; a File's bytes are shared, not copied). */
  files: File[];
}

export type ModelResponse = { id: number; scene: SceneData } | { id: number; error: string };
