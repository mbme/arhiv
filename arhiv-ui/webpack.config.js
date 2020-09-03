const path = require('path') // eslint-disable-line
const webpack = require('webpack') // eslint-disable-line

const mode = process.env.NODE_ENV || 'development'
const isProduction = mode === 'production'

module.exports = {
  mode,

  entry: './web/index.tsx',

  output: {
    filename: 'bundle.js',
    path: path.resolve(__dirname, 'dist'),
  },

  resolve: {
    extensions: [ '.tsx', '.ts', '.js' ],
  },

  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
      },
    ],
  },

  plugins: [
    new webpack.DefinePlugin({ // eslint-disable-line
      'process.env.NODE_ENV': JSON.stringify(mode),
    }),
  ],
}
