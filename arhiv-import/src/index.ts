import puppeteer from 'puppeteer-core';

import { ActionChannel } from './ActionChannel';
import type { Importer } from './utils';
import { extractBookFromYakaboo } from './book-yakaboo';
import { extractFilmFromIMDB } from './film-imdb';

const channel = new ActionChannel();

const SCRAPERS: Importer[] = [
  extractBookFromYakaboo,
  extractFilmFromIMDB,
];

async function scrape(url: string, debug: boolean): Promise<void> {
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
      const success = await scraper(url, browser, channel);

      if (success) {
        return;
      }
    }

    throw new Error('No matching scraper');
  } finally {
    await browser.close();
  }
}

const args = process.argv.slice(2);
const url = args[0];
const debug = args[1] === 'true';

scrape(url, debug).then(
  () => {
    channel.close();
  },
  (e) => {
    console.error('Failed to scrape url %s', url, e);
    process.exit(1);
  },
);
