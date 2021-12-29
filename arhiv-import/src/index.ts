import { Context } from './context';
import { extractBookFromYakaboo } from './book-yakaboo';
import { extractFilmFromIMDB } from './film-imdb';
import { extractImage } from './attachment-image';

type Importer = (url: string, context: Context) => Promise<boolean>;

const args = process.argv.slice(2);
const url = args[0];
const debug = args[1] === 'true';

const context = new Context(debug);

const SCRAPERS: Importer[] = [
  extractBookFromYakaboo,
  extractFilmFromIMDB,
  extractImage,
];

async function scrape(url: string): Promise<void> {
  if (!url) {
    throw new Error('url to scrape must be provided');
  }

  for (const scraper of SCRAPERS) {
    const success = await scraper(url, context);

    if (success) {
      return;
    }
  }

  throw new Error('No matching scraper');
}

async function run(url: string) {
  try {
    await scrape(url);
  } catch (e) {
    console.error('Failed to scrape url %s', url, e);
    throw e;
  } finally {
    await context.close();
  }
}

run(url).catch(
  (e) => {
    console.error('Failed to run: ', e);
    process.exit(1);
  },
);
