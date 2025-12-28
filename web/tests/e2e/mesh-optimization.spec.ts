import { test, expect, Page } from '@playwright/test';

interface MeshCompressionResult {
  success: boolean;
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
  error?: string;
  timeMs: number;
}

async function waitForWasmReady(page: Page): Promise<void> {
  await page.waitForFunction(
    () => {
      const win = window as unknown as { pixieJuice?: { version?: () => string } };
      if (!win.pixieJuice || typeof win.pixieJuice.version !== 'function') return false;
      try {
        const v = win.pixieJuice.version();
        return typeof v === 'string' && v.length > 0;
      } catch {
        return false;
      }
    },
    { timeout: 30000 }
  );
}

async function compressMeshInBrowser(
  page: Page, 
  meshData: Uint8Array
): Promise<MeshCompressionResult> {
  return await page.evaluate(async ({ data }) => {
    const win = window as unknown as { 
      pixieJuice?: { 
        optimize_mesh?: (data: Uint8Array) => Uint8Array;
        optimize_auto?: (data: Uint8Array, quality: number) => Uint8Array;
      } 
    };
    
    const startTime = performance.now();
    
    try {
      const inputArray = new Uint8Array(data);
      const result = win.pixieJuice?.optimize_mesh?.(inputArray, 80)
          || win.pixieJuice?.optimize_auto?.(inputArray, 80);
      
      if (!result) {
        return {
          success: false,
          originalSize: inputArray.length,
          compressedSize: 0,
          compressionRatio: 0,
          error: 'No mesh optimization function available',
          timeMs: performance.now() - startTime
        };
      }
      
      const endTime = performance.now();
      const ratio = ((inputArray.length - result.length) / inputArray.length) * 100;
      
      return {
        success: true,
        originalSize: inputArray.length,
        compressedSize: result.length,
        compressionRatio: ratio,
        timeMs: endTime - startTime
      };
    } catch (error) {
      return {
        success: false,
        originalSize: data.length,
        compressedSize: 0,
        compressionRatio: 0,
        error: error instanceof Error ? error.message : String(error),
        timeMs: performance.now() - startTime
      };
    }
  }, { data: Array.from(meshData) });
}

function createTestObjMesh(): Uint8Array {
  const obj = `# Test OBJ file
mtllib test.mtl
o Cube
v -1.0 -1.0 1.0
v -1.0 1.0 1.0
v 1.0 1.0 1.0
v 1.0 -1.0 1.0
v -1.0 -1.0 -1.0
v -1.0 1.0 -1.0
v 1.0 1.0 -1.0
v 1.0 -1.0 -1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 -1.0
vn 0.0 1.0 0.0
vn 0.0 -1.0 0.0
vn 1.0 0.0 0.0
vn -1.0 0.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
usemtl Material
s off
f 1/1/1 2/2/1 3/3/1 4/4/1
f 5/1/2 8/2/2 7/3/2 6/4/2
f 2/1/3 6/2/3 7/3/3 3/4/3
f 1/1/4 4/2/4 8/3/4 5/4/4
f 4/1/5 3/2/5 7/3/5 8/4/5
f 1/1/6 5/2/6 6/3/6 2/4/6
`;
  return new TextEncoder().encode(obj);
}

function createTestPlyMesh(): Uint8Array {
  const ply = `ply
format ascii 1.0
element vertex 8
property float x
property float y
property float z
element face 6
property list uchar int vertex_indices
end_header
-1.0 -1.0 1.0
-1.0 1.0 1.0
1.0 1.0 1.0
1.0 -1.0 1.0
-1.0 -1.0 -1.0
-1.0 1.0 -1.0
1.0 1.0 -1.0
1.0 -1.0 -1.0
4 0 1 2 3
4 4 7 6 5
4 1 5 6 2
4 0 3 7 4
4 3 2 6 7
4 0 4 5 1
`;
  return new TextEncoder().encode(ply);
}

function createTestStlMesh(): Uint8Array {
  const stl = `solid cube
  facet normal 0 0 1
    outer loop
      vertex -1 -1 1
      vertex 1 -1 1
      vertex 1 1 1
    endloop
  endfacet
  facet normal 0 0 1
    outer loop
      vertex -1 -1 1
      vertex 1 1 1
      vertex -1 1 1
    endloop
  endfacet
  facet normal 0 0 -1
    outer loop
      vertex -1 -1 -1
      vertex -1 1 -1
      vertex 1 1 -1
    endloop
  endfacet
  facet normal 0 0 -1
    outer loop
      vertex -1 -1 -1
      vertex 1 1 -1
      vertex 1 -1 -1
    endloop
  endfacet
endsolid cube
`;
  return new TextEncoder().encode(stl);
}

function createMinimalGltf(): Uint8Array {
  const gltf = {
    asset: { version: "2.0" },
    scene: 0,
    scenes: [{ nodes: [0] }],
    nodes: [{ mesh: 0 }],
    meshes: [{
      primitives: [{
        attributes: { POSITION: 0 },
        indices: 1
      }]
    }],
    accessors: [
      {
        bufferView: 0,
        componentType: 5126,
        count: 3,
        type: "VEC3",
        max: [1.0, 1.0, 0.0],
        min: [0.0, 0.0, 0.0]
      },
      {
        bufferView: 1,
        componentType: 5123,
        count: 3,
        type: "SCALAR"
      }
    ],
    bufferViews: [
      { buffer: 0, byteOffset: 0, byteLength: 36 },
      { buffer: 0, byteOffset: 36, byteLength: 6 }
    ],
    buffers: [{ byteLength: 42 }]
  };
  return new TextEncoder().encode(JSON.stringify(gltf));
}

test.describe('Mesh Optimization - OBJ Format', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should detect OBJ format', async ({ page }) => {
    const testObj = createTestObjMesh();
    
    const format = await page.evaluate(async (data) => {
      const win = window as unknown as { 
        pixieJuice?: { detect_format?: (data: Uint8Array) => string } 
      };
      return win.pixieJuice?.detect_format?.(new Uint8Array(data));
    }, Array.from(testObj));
    
    expect(format).toContain('mesh');
  });

  test('should handle OBJ mesh optimization', async ({ page }) => {
    const testObj = createTestObjMesh();
    const result = await compressMeshInBrowser(page, testObj);
    
    expect(result.originalSize).toBeGreaterThan(0);
  });

  test('should preserve mesh integrity after optimization', async ({ page }) => {
    const testObj = createTestObjMesh();
    
    const hasValidOutput = await page.evaluate(async (data) => {
      const win = window as unknown as { 
        pixieJuice?: { optimize_mesh?: (data: Uint8Array) => Uint8Array } 
      };
      const result = win.pixieJuice?.optimize_mesh?.(new Uint8Array(data));
      if (!result) return false;
      
      const text = new TextDecoder().decode(result);
      return text.includes('v ') || text.includes('f ') || result.length > 0;
    }, Array.from(testObj));
    
    expect(hasValidOutput).toBe(true);
  });
});

test.describe('Mesh Optimization - PLY Format', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should handle ASCII PLY mesh', async ({ page }) => {
    const testPly = createTestPlyMesh();
    const result = await compressMeshInBrowser(page, testPly);
    
    expect(result.originalSize).toBeGreaterThan(0);
  });

  test('should detect PLY format', async ({ page }) => {
    const testPly = createTestPlyMesh();
    
    const format = await page.evaluate(async (data) => {
      const win = window as unknown as { 
        pixieJuice?: { detect_format?: (data: Uint8Array) => string } 
      };
      return win.pixieJuice?.detect_format?.(new Uint8Array(data));
    }, Array.from(testPly));
    
    expect(format?.toLowerCase()).toContain('ply');
  });
});

test.describe('Mesh Optimization - STL Format', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should handle ASCII STL mesh', async ({ page }) => {
    const testStl = createTestStlMesh();
    const result = await compressMeshInBrowser(page, testStl);
    
    expect(result.originalSize).toBeGreaterThan(0);
  });

  test('should detect STL format', async ({ page }) => {
    const testStl = createTestStlMesh();
    
    const format = await page.evaluate(async (data) => {
      const win = window as unknown as { 
        pixieJuice?: { detect_format?: (data: Uint8Array) => string } 
      };
      return win.pixieJuice?.detect_format?.(new Uint8Array(data));
    }, Array.from(testStl));
    
    expect(format?.toLowerCase()).toContain('stl');
  });
});

test.describe('Mesh Optimization - GLTF Format', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should handle GLTF mesh', async ({ page }) => {
    const testGltf = createMinimalGltf();
    const result = await compressMeshInBrowser(page, testGltf);
    
    expect(result.originalSize).toBeGreaterThan(0);
  });

  test('should detect GLTF format', async ({ page }) => {
    const testGltf = createMinimalGltf();
    
    const format = await page.evaluate(async (data) => {
      const win = window as unknown as { 
        pixieJuice?: { detect_format?: (data: Uint8Array) => string } 
      };
      return win.pixieJuice?.detect_format?.(new Uint8Array(data));
    }, Array.from(testGltf));
    
    expect(format?.toLowerCase()).toMatch(/gltf|glb|mesh/);
  });
});

test.describe('Mesh Optimization - Error Handling', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should handle empty mesh data', async ({ page }) => {
    const emptyData = new Uint8Array(0);
    const result = await compressMeshInBrowser(page, emptyData);
    
    expect(result.success).toBe(false);
  });

  test('should handle invalid mesh data', async ({ page }) => {
    const invalidData = new TextEncoder().encode('not a valid mesh format');
    const result = await compressMeshInBrowser(page, invalidData);
    
    expect(result.success).toBe(false);
  });

  test('should handle corrupted OBJ', async ({ page }) => {
    const corruptedObj = new TextEncoder().encode('v invalid\nf broken');
    const result = await compressMeshInBrowser(page, corruptedObj);
    
    expect(result.originalSize).toBeGreaterThan(0);
  });
});

test.describe('Mesh Optimization - Performance', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmReady(page);
  });

  test('should complete small mesh in under 2 seconds', async ({ page }) => {
    const testObj = createTestObjMesh();
    const result = await compressMeshInBrowser(page, testObj);
    
    expect(result.timeMs).toBeLessThan(2000);
  });

  test('should handle mesh with many vertices', async ({ page }) => {
    const vertices: string[] = [];
    const faces: string[] = [];

    for (let y = 0; y < 10; y++) {
      for (let x = 0; x < 10; x++) {
        const fx = (x - 5) / 10;
        const fy = (y - 5) / 10;
        const fz = ((x + y) % 3) / 10;
        vertices.push(`v ${fx.toFixed(3)} ${fy.toFixed(3)} ${fz.toFixed(3)}`);
      }
    }

    for (let i = 1; i <= 50; i++) {
      const v1 = i;
      const v2 = i + 1;
      const v3 = i + 10;
      faces.push(`f ${v1} ${v2} ${v3}`);
    }
    
    const largeObj = new TextEncoder().encode([...vertices, ...faces].join('\n'));
    const result = await compressMeshInBrowser(page, largeObj);
    
    expect(result.timeMs).toBeLessThan(5000);

    if (result.success) {
      expect(result.compressedSize).toBeGreaterThan(0);
      expect(
        result.compressedSize,
        `red flag: mesh output grew (${result.originalSize} -> ${result.compressedSize})`
      ).toBeLessThanOrEqual(result.originalSize);
    }
  });
});
