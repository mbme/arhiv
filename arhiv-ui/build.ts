/* eslint-env node */

import esbuild, { BuildOptions } from 'esbuild';

const isProduction = process.env.NODE_ENV === 'production';
const watch = process.argv.includes('--watch');

const options: BuildOptions = {
  entryPoints: {
    'index': './src/ui/index.tsx',
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
};

if (watch) {
  const context = await esbuild.context(options);
  await context.watch();
} else {
  await esbuild.build(options);
}
