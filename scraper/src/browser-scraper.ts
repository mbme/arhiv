import { Scraper } from './scraper';

import { scrapeFBPost } from './facebook/post';
import { scrapeFBPostList } from './facebook/post-list';
import { scrapeFBMobilePostList } from './facebook/post-list-mobile';
import { scrapeFBMobilePost } from './facebook/post-mobile';

const SCRAPERS: Scraper[] = [
  //
  scrapeFBPost,
  scrapeFBPostList,
  scrapeFBMobilePost,
  scrapeFBMobilePostList,
];

async function scrape(): Promise<unknown | undefined> {
  for (const scraper of SCRAPERS) {
    try {
      const result = await scraper();

      if (result) {
        console.info(`scraper ${scraper.name} succeeded`);
        return result;
      }
    } catch (e) {
      console.error(`scraper ${scraper.name} failed:`, e);
      throw e;
    }
  }

  return undefined;
}

declare global {
  interface Window {
    originalURL: URL;
    _scrape: () => Promise<unknown | undefined>;
    _onScrape?: (result: unknown | null, error: string | null) => void;
  }
}

window._scrape = async () => {
  try {
    const result = await scrape();

    window._onScrape?.(result, null);

    return result;
  } catch (e) {
    window._onScrape?.(null, (e as Error).toString());

    throw e;
  }
};
