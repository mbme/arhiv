import nodeResolve from 'rollup-plugin-node-resolve';
import commonjs from 'rollup-plugin-commonjs';
import replace from 'rollup-plugin-replace';
import typescript from 'rollup-plugin-typescript';
import { terser } from 'rollup-plugin-terser';
import rollupPluginCss from './rollup-plugin-css';

const isProduction = process.env.NODE_ENV === 'production';

export default {
  input: 'src/web-app/index.tsx',

  output: {
    file: 'dist/bundle.js',
    format: 'iife',
    sourcemap: true,
  },

  plugins: [
    nodeResolve({
      jsnext: true,
      extensions: [ '.mjs', '.js', '.jsx', '.ts', '.tsx' ],
    }),

    rollupPluginCss(),

    typescript(),

    replace({
      'process.env.NODE_ENV': JSON.stringify('production'),
      'process.env.LOG': JSON.stringify(process.env.LOG),
      __SERVER__: JSON.stringify(false),
    }),

    commonjs({
      namedExports: {
        'node_modules/react/index.js': [
          'Component',
          'PureComponent',
          'Fragment',
          'StrictMode',
          'createRef',
          'createElement',
          'render',
        ],
        'node_modules/react-dom/index.js': [
          'createElement',
          'render',
        ],
      },
    }),

    isProduction && terser(),
  ],

  watch: {
    clearScreen: false,
  },
};
