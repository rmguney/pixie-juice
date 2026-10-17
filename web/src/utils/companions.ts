// The files a .gltf refers to by URI (its .bin buffers, its textures), found
// among the other files in the list by name: the engine packs them into the
// GLB, and the preview reads them, instead of either one fetching anything.
// The glTF itself is only parsed in the workers.

import { fileKind } from './files';

export interface GltfJson {
  buffers?: { uri?: string }[];
  images?: { uri?: string }[];
}

/** The glTF JSON of a .gltf file, or of a GLB's JSON chunk. */
export function gltfJson(bytes: Uint8Array): { glb: boolean; json: GltfJson } {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const glb = bytes.length >= 20 && view.getUint32(0, true) === 0x46546c67; // "glTF"
  const text = glb ? bytes.subarray(20, 20 + view.getUint32(12, true)) : bytes;
  return { glb, json: JSON.parse(new TextDecoder().decode(text)) };
}

/** The URIs of a glTF's buffers and images that name other files (not data: URIs), once each. */
export function externalUris(json: GltfJson): string[] {
  const uris = [...(json.buffers ?? []), ...(json.images ?? [])].map((item) => item.uri);
  return [...new Set(uris.filter((uri): uri is string => typeof uri === 'string' && !uri.startsWith('data:')))];
}

/** The file name a glTF URI points at: decoded, without folders. */
export function uriFileName(uri: string): string {
  let decoded = uri;
  try {
    decoded = decodeURIComponent(uri);
  } catch {
    // a malformed escape: the name as written
  }
  return decoded.slice(Math.max(decoded.lastIndexOf('/'), decoded.lastIndexOf('\\')) + 1);
}

/** The files in `list` a .gltf might refer to (anything but meshes); none for other files. */
export function companionCandidates(file: File, list: File[]): File[] {
  if (!file.name.toLowerCase().endsWith('.gltf')) return [];
  return list.filter((candidate) => candidate !== file && fileKind(candidate.name) !== 'mesh');
}
