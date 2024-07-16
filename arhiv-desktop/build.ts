/* eslint-env node */

import esbuild from 'esbuild';
import fs from 'node:fs/promises';

await fs.rm('./dist', { recursive: true, force: true });

const isProduction = process.env.NODE_ENV === 'production';

await esbuild.build({
  entryPoints: ['./src/index.ts'],
  outfile: './dist/index.cjs',

  banner: {
    js: '#!/usr/bin/env electron',
  },

  format: 'cjs',
  platform: 'node',
  target: ['node18.12'],
  external: ['electron'],
  loader: { '.png': 'dataurl' },

  define: {
    'process.env.NODE_ENV': isProduction ? '"production"' : '"development"',
  },

  bundle: true,
  minify: false, // ease of debugging is more important than size
  sourcemap: true,
});
