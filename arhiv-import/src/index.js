/* eslint-env node */

const puppeteer = require('puppeteer-core');
const { extractBookFromYakaboo } = require('./book-yakaboo');

const SCRAPERS = [
  extractBookFromYakaboo,
];

async function scrape(url) {
  if (!url) {
    throw new Error('url to scrape must be provided');
  }

  const browser = await puppeteer.launch({
    headless: false,
    executablePath: '/usr/bin/chromium', // FIXME run `which chromium`
    defaultViewport: null, // use full viewport FIXME disable in prod
  });

  try {
    for (const scraper of SCRAPERS) {
      const result = await scraper(url, browser);

      if (result) {
        return result;
      }
    }
  } finally {
    await browser.close();
  }
}

async function run(url) {
  const data = await scrape(url);

  if (!data) {
    console.error('No scraper for url %s', url);
    return;
  }

  console.log(data);
}

const args = process.argv.slice(2);
const url = args[0];

run(url).catch((e) => {
  console.error('Failed to scrape url %s', url, e);
  process.exit(1);
});
