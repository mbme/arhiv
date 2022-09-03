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

  target: ['es2020'],
  bundle: true,
  minify: isProduction,
  sourcemap: true,

  jsxFactory: 'h',
  jsxFragment: 'Fragment',
  jsxImportSource: 'preact',

  loader: {
    '.html': 'text',
  },

  define: {
    'process.env.NODE_ENV': isProduction ? '"production"' : '"development"',
  },

  watch,
});
