import path from 'path';
import nodeResolve from 'rollup-plugin-node-resolve'
import commonjs from 'rollup-plugin-commonjs'
import replace from '@rollup/plugin-replace'

const isProduction = process.env.NODE_ENV === 'production'

export const tsPathResolver = (rootDir) => ({
  resolveId(importee) {
    if (importee.startsWith('~/')) {
      return this.resolveId(path.resolve(rootDir, importee.substring(2)));
    }

    return null;
  }

})

const BASE_DIR = process.env.BASE_DIR

export default {
  input: `${BASE_DIR}/web-app/index`,

  output: {
    file: `${BASE_DIR}/bundle.js`,
    format: 'iife',
    name: 'WebApp',
  },

  treeshake: isProduction,

  plugins: [
    tsPathResolver(BASE_DIR),

    nodeResolve(),

    replace({
      'process.env.NODE_ENV': JSON.stringify(isProduction ? 'production' : 'development'),
      'process.env.__BROWSER__': true,
    }),

    commonjs(),
  ],

  watch: {
    clearScreen: false,
    include: `${BASE_DIR}/**`,
    exclude: ['node_modules/**'],
  },
}
