/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config, { isServer }) => {
    // Handle WASM files with proper support
    config.experiments = {
      ...config.experiments,
      syncWebAssembly: true,
      // Remove asyncWebAssembly to avoid warnings
      layers: true,
    };
    
    // Add rule for .wasm files
    config.module.rules.push({
      test: /\.wasm$/,
      type: 'webassembly/sync',
    });
    
    // Handle imports and fallbacks for client-side
    config.resolve.fallback = {
      ...config.resolve.fallback,
      fs: false,
      path: false,
      crypto: false,
      env: false,
      util: false,
      assert: false,
      buffer: false,
      stream: false,
    };
    
    // Ignore specific modules that WASM tries to import
    config.resolve.alias = {
      ...config.resolve.alias,
      env: false,
    };
    
    // Additional WASM configuration
    config.optimization = {
      ...config.optimization,
      moduleIds: 'deterministic',
    };
    
    return config;
  },
};

export default nextConfig;
