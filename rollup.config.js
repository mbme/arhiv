import path from 'path';
import nodeResolve from 'rollup-plugin-node-resolve'
import commonjs from 'rollup-plugin-commonjs'
import replace from 'rollup-plugin-replace'

const isProduction = process.env.NODE_ENV === 'production'

const aliasPlugin = {
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

  moduleContext: {  // HACK: suppress useless rollup warning for 3rd party library
    'node_modules/free-style/dist.es2015/free-style.js': 'window',
  },

  plugins: [
    aliasPlugin,

    nodeResolve(),

    replace({
      'process.env.NODE_ENV': JSON.stringify(isProduction ? 'production' : 'development'),
      'process.env.LOG': JSON.stringify(process.env.LOG),
    }),

    commonjs({
      namedExports: {
        'node_modules/react/index.js': [
          'Component',
          'PureComponent',
          'Fragment',
          'StrictMode',
          'createRef',
          'createContext',
          'createElement',
          'useEffect',
          'useContext',
          'useState',
          'useRef',
          'useMemo',
          'memo',
          'render',
        ],
        'node_modules/react-dom/index.js': [
          'createElement',
          'render',
        ],
      },
    }),
  ],

  watch: {
    clearScreen: false,
    exclude: 'node_modules/**',
  },
}
