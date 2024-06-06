/* eslint-env node */

import esbuild from 'esbuild';
import fs from 'node:fs/promises';

await fs.rm('./dist', { recursive: true, force: true });

await esbuild.build({
  entryPoints: ['./src/index.ts'],
  outfile: './dist/index.cjs',

  format: 'cjs',
  platform: 'node',
  target: ['node18.12'],
  external: ['electron'],
  loader: { '.png': 'dataurl' },

  bundle: true,
  minify: false, // ease of debugging is more important than size
  sourcemap: true,
});

await esbuild.build({
  entryPoints: ['./src/preload.ts'],
  outfile: './dist/preload.cjs',

  format: 'cjs',
  platform: 'node',
  target: ['node18.12'],

  bundle: false,
  minify: false, // ease of debugging is more important than size
  sourcemap: true,
});
