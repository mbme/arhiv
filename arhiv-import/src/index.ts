import puppeteer from 'puppeteer-core';

import type { Obj } from './utils';
import { extractBookFromYakaboo } from './book-yakaboo';

const SCRAPERS = [
  extractBookFromYakaboo,
];

async function scrape(url: string): Promise<Obj | undefined> {
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

    return undefined;
  } finally {
    await browser.close();
  }
}

const args = process.argv.slice(2);
const url = args[0];

scrape(url).then((data) => {
  if (!data) {
    throw new Error(`No scraper for url ${url}`);
  }

  process.stdout.write(JSON.stringify(data, null, 2));
}).catch((e) => {
  console.error('Failed to scrape url %s', url, e);
  process.exit(1);
});
