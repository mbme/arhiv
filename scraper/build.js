/* eslint-env node */
/* eslint-disable @typescript-eslint/no-var-requires */

const fs = require('fs');
const esbuild = require('esbuild');

const isProduction = process.env.NODE_ENV === 'production';

const BROWSER_SCRAPER_OUT_FILE = './dist/browser-scraper.js';
const NODE_SCRAPER_OUT_FILE = './dist/node-scraper.js';

async function buildBrowserBundle() {
  await esbuild.build({
    entryPoints: ['./src/browser-scraper.ts'],
    outfile: BROWSER_SCRAPER_OUT_FILE,

    target: ['es2020', 'chrome90', 'firefox87'],
    bundle: true,
    minify: isProduction,
    sourcemap: true,

    define: {
      'process.env.NODE_ENV': isProduction ? '"production"' : '"development"',
    },
  });
}

async function buildNodeBundle() {
  const browserScraper = await fs.promises.readFile(BROWSER_SCRAPER_OUT_FILE);

  await esbuild.build({
    entryPoints: ['./src/node-scraper.ts'],
    outfile: NODE_SCRAPER_OUT_FILE,

    bundle: true,
    platform: 'node',

    minify: isProduction,
    sourcemap: true,

    // Needed for puppeteer https://github.com/evanw/esbuild/issues/1492#issuecomment-893144483
    inject: ['./import-meta-url.js'],

    define: {
      '__BROWSER_SCRAPER__': JSON.stringify(browserScraper.toString()),
      'process.env.NODE_ENV': isProduction ? '"production"' : '"development"',
      'import.meta.url': 'import_meta_url',
    },
  });
}

async function build() {
  await fs.promises.rm('./dist', { force: true, recursive: true });

  await buildBrowserBundle();
  await buildNodeBundle();
}

void build();
