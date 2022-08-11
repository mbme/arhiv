/* eslint-env node */
/* eslint-disable @typescript-eslint/no-var-requires */

const esbuild = require('esbuild');

const isProduction = process.env.NODE_ENV === 'production';
const watch = process.argv.includes('--watch');

void esbuild.build({
  entryPoints: {
    'index': './src/index.ts',
    'workspace': './src/workspace/index.tsx',
  },
  outdir: './public',

  target: ['es2020', 'chrome90', 'firefox87'],
  bundle: true,
  minify: isProduction,
  sourcemap: true,

  inject: ['./preact-shim.js'],
  jsxFactory: 'h',
  jsxFragment: 'Fragment',

  loader: {
    '.html': 'text',
  },

  define: {
    'process.env.NODE_ENV': isProduction ? '"production"' : '"development"',
  },

  watch,
});
