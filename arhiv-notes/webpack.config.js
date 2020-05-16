// eslint-disable-next-line
const path = require('path')

module.exports = {
  mode: process.env.NODE_ENV || 'development',
  entry: './web/index.tsx',
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
