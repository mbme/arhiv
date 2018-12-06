import { Config } from '@stencil/core';

// https://stenciljs.com/docs/config

const APP_DIR = 'src/web-app'
export const config: Config = {
  globalStyle: `${APP_DIR}/global/app.css`,
  globalScript: `${APP_DIR}/global/app.ts`,
  srcDir: APP_DIR,
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
