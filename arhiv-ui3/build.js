/* eslint-env node */

const esbuild = require('esbuild');

const isProduction = process.env.NODE_ENV === 'production';
const watch = process.argv.includes('--watch');

void esbuild.build({
  entryPoints: ['./src/index.ts'],
  outfile: './public/index.js',

  target: ['es2020', 'chrome90', 'firefox87'],
  bundle: true,
  minify: isProduction,
  sourcemap: true,

  inject: ['./preact-shim.js'],

  loader: {
    '.html': 'text',
  },

  define: {
    'process.env.NODE_ENV': isProduction ? '"production"' : '"development"',
  },

  watch,
});
