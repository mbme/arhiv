import path from 'path';
import nodeResolve from '@rollup/plugin-node-resolve '
import commonjs from 'rollup-plugin-commonjs'
import replace from '@rollup/plugin-replace'
import React from 'react'
import ReactDOM from 'react-dom'

const isProduction = process.env.NODE_ENV === 'production'

const tsPathResolver = (rootDir) => ({
  resolveId(importee) {
    if (importee.startsWith('~/')) {
      return this.resolveId(path.resolve(rootDir, importee.substring(2)));
    }

    return null;
  }

})

const BASE_DIR = process.env.BASE_DIR

const plugins = [
  tsPathResolver(BASE_DIR),

  nodeResolve({
    preferBuiltins: false,
  }),

  replace({
    'process.env.NODE_ENV': JSON.stringify(isProduction ? 'production' : 'development'),
    'process.env.__BROWSER__': true,
  }),

  commonjs({
    include: 'node_modules/**',

    namedExports: {
      'react': Object.keys(React),
      'react-dom': Object.keys(ReactDOM)
    },
  }),
]

export default [
  {
    input: `${BASE_DIR}/web-app/index`,

    output: {
      file: `${BASE_DIR}/bundle.js`,
      format: 'iife',
      name: 'WebApp',
      globals: {
        'crypto': 'crypto', // use window.crypto in browser
      },
    },
    external: ['crypto'],

    treeshake: isProduction,

    plugins,

    watch: {
      clearScreen: false,
      include: `${BASE_DIR}/**`,
      exclude: ['node_modules/**'],
    },
  },

  {
    input: `${BASE_DIR}/web-app/serviceWorker`,

    output: {
      file: `${BASE_DIR}/serviceWorker.js`,
      format: 'iife',
      name: 'WebAppServiceWorker',
    },

    treeshake: isProduction,

    plugins,
  }
]
