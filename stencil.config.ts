import { Config } from '@stencil/core';

// https://stenciljs.com/docs/config

export const config: Config = {
  globalStyle: 'src/isodb-web-client/global/app.css',
  globalScript: 'src/isodb-web-client/global/app.ts',
  srcDir: 'src/isodb-web-client',
  outputTargets: [
    {
      type: 'www',
      buildDir: 'dist',
      dir: 'dist',
    }
  ],
  devServer: {
    openBrowser: false,
  },
};
