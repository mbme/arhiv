// eslint-disable-next-line
const path = require('path')

const isProduction = process.env.NODE_ENV === 'production'

module.exports = {
  mode: process.env.NODE_ENV || 'development',
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
}
