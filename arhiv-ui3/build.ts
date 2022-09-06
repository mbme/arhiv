/* eslint-env node */

import esbuild from 'esbuild';

const isProduction = process.env.NODE_ENV === 'production';
const watch = process.argv.includes('--watch');

await esbuild.build({
  entryPoints: {
    'index': './src/index.ts',
    'workspace': './src/workspace/index.tsx',
  },
  outdir: './public',

  target: ['es2020'],
  bundle: true,
  minify: isProduction,
  sourcemap: true,

  jsxImportSource: 'preact',
  jsx: 'automatic',

  loader: {
    '.html': 'text',
  },

  define: {
    'process.env.NODE_ENV': isProduction ? '"production"' : '"development"',
  },

  watch,
});
