const path = require('path') // eslint-disable-line
const webpack = require('webpack') // eslint-disable-line

const mode = process.env.NODE_ENV || 'development'
const isProduction = mode === 'production'

module.exports = {
  mode,
  entry: [
    isProduction ? null : 'react-devtools',
    './web/index.tsx',
  ].filter(Boolean),
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
      },
    ],
  },
  resolve: {
    extensions: [ '.tsx', '.ts', '.js' ],
  },
  output: {
    filename: 'bundle.js',
    path: path.resolve(__dirname, 'static'),
  },
  plugins: [
    new webpack.DefinePlugin({ // eslint-disable-line
      'process.env.NODE_ENV': JSON.stringify(mode),
    }),
  ]
}
