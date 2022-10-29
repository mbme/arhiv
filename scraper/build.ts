/* eslint-env node */

import esbuild from 'esbuild';

const isProduction = process.env.NODE_ENV === 'production';

await esbuild.build({
  entryPoints: ['./src/browser-scraper.ts'],
  outfile: './dist/browser-scraper.js',

  target: ['es2020'],
  bundle: true,
  minify: isProduction,
  sourcemap: true,

  define: {
    'process.env.NODE_ENV': isProduction ? '"production"' : '"development"',
  },
});
