/* eslint-env node */

const esbuild = require('esbuild');

const isProduction = process.env.NODE_ENV === 'production';
const watch = process.argv.includes('--watch');

esbuild.build({
  outdir: './public',
  entryPoints: {
    index: './src/index.ts',
    unpoly: 'unpoly/unpoly.css',
  },

  target: [
    'es2020',
    'chrome90',
    'firefox87',
  ],
  bundle: true,
  minify: isProduction,
  sourcemap: true,

  define: {
    'process.env.NODE_ENV': isProduction ? '"production"' : '"development"',
  },

  watch,
});
