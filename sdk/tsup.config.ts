import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  splitting: true,
  sourcemap: true,
  clean: true,
  treeshake: true,
  minify: false,
  target: 'es2022',
  outDir: 'dist',
  platform: 'neutral',
  external: [],
  noExternal: ['eventemitter3'],
  banner: {
    js: '/* @llm-cost-ops/sdk - Enterprise-grade TypeScript SDK for LLM Cost Operations */',
  },
  esbuildOptions(options) {
    options.conditions = ['module', 'import', 'require', 'default'];
  },
});
