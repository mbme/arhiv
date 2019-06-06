import nodeResolve from 'rollup-plugin-node-resolve'
import commonjs from 'rollup-plugin-commonjs'
import replace from 'rollup-plugin-replace'
import typescript from 'rollup-plugin-typescript2'

const isProduction = process.env.NODE_ENV === 'production'

export default {
  input: 'src/web-app/index.tsx',

  output: {
    file: 'dist/bundle.js',
    format: 'iife',
    sourcemap: isProduction,
  },

  treeshake: isProduction,

  moduleContext: {  // HACK: suppress useless rollup warning for 3rd party library
    'node_modules/free-style/dist.es2015/free-style.js': 'window',
  },

  plugins: [
    nodeResolve({
      extensions: [ '.mjs', '.js', '.jsx', '.ts', '.tsx' ],
    }),

    typescript({
      /* cacheRoot: './node_modules/.cache/rts2_cache', */
      cacheRoot: require('os').tmpdir() + '/rts2_cache',
    }),

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
