import path from 'path';
import nodeResolve from 'rollup-plugin-node-resolve'
import commonjs from 'rollup-plugin-commonjs'
import replace from 'rollup-plugin-replace'

const isProduction = process.env.NODE_ENV === 'production'

const tsPathResolver = {
  resolveId(importee) {
    if (importee.startsWith('~/')) {
      return this.resolveId(path.resolve(__dirname, 'tsdist', importee.substring(2)));
    }

    return null;
  }

}

export default {
  input: 'tsdist/web-app/index',

  output: {
    file: 'dist/bundle.js',
    format: 'iife',
    name: 'WebApp',
  },

  treeshake: isProduction,

  plugins: [
    tsPathResolver,

    nodeResolve(),

    replace({
      'process.env.NODE_ENV': JSON.stringify(isProduction ? 'production' : 'development'),
      'process.env.LOG': JSON.stringify(process.env.LOG),
      '__BROWSER__': true,
    }),

    commonjs(),
  ],

  watch: {
    clearScreen: false,
    include: 'tsdist/**',
    exclude: ['node_modules/**'],
  },
}
