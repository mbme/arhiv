// eslint-disable-next-line
const path = require('path')

module.exports = {
  mode: 'development',
  entry: './src/LibraryApp.tsx',
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
    path: path.resolve(__dirname, 'dist'),
  },
  devServer: {
    contentBase: path.join(__dirname, 'static'),
    port: 8080,
    hot: true,
  },
}
