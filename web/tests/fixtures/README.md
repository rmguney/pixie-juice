# Test Fixtures

This directory contains test fixtures used by the test suite.

## Files

- `regression-baseline.json` - Baseline data for regression tests. Update with `UPDATE_BASELINE=true npm run test:e2e`

## Adding Test Images

For testing with real image files, place them in this directory:

```
fixtures/
  images/
    test-small.png
    test-large.jpg
    test-animated.gif
  meshes/
    test-cube.obj
    test-model.gltf
```

The test suite also generates synthetic test data programmatically to avoid storing binary files in version control.
