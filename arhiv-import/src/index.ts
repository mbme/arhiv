import puppeteer from 'puppeteer-core';

import type { Importer, Obj } from './utils';
import { extractBookFromYakaboo } from './book-yakaboo';
import { extractFilmFromIMDB } from './film-imdb';

const SCRAPERS: Importer[] = [
  extractBookFromYakaboo,
  extractFilmFromIMDB,
];

async function scrape(url: string, debug: boolean): Promise<Obj | undefined> {
  if (!url) {
    throw new Error('url to scrape must be provided');
  }

  const executablePath = '/usr/bin/chromium'; // FIXME run `which chromium`
  const browser = await puppeteer.launch(debug ? {
    executablePath,
    headless: false,
    defaultViewport: null, // use full viewport
  } : {
    executablePath,
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

const isProduction = process.env.NODE_ENV === 'production';

scrape(url, !isProduction).then((data) => {
  if (!data) {
    throw new Error(`No scraper for url ${url}`);
  }

  process.stdout.write(JSON.stringify(data, null, 2));
}).catch((e) => {
  console.error('Failed to scrape url %s', url, e);
  process.exit(1);
});
