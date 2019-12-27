import path from 'path';
import nodeResolve from '@rollup/plugin-node-resolve '
import commonjs from '@rollup/plugin-commonjs'
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

const baseDir = path.join(process.cwd(), 'tsdist/src')
const webAppStatic = path.join(process.cwd(), 'tsdist/web-app-static')

const plugins = [
  tsPathResolver(baseDir),

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
    input: `${baseDir}/web-app/index`,

    output: {
      file: `${webAppStatic}/bundle.js`,
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
      include: `${baseDir}/**`,
      exclude: ['node_modules/**'],
    },
  },

  {
    input: `${baseDir}/web-app/serviceWorker`,

    output: {
      file: `${webAppStatic}/serviceWorker.js`,
      format: 'iife',
      name: 'WebAppServiceWorker',
    },

    treeshake: isProduction,

    plugins,
  }
]
