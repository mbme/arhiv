import { ExtractScraperGeneric } from './scraper';

import { scrapeFBPost } from './facebook/post';
import { scrapeFBPostList } from './facebook/post-list';
import { scrapeFBMobilePostList } from './facebook/post-list-mobile';
import { scrapeFBMobilePost } from './facebook/post-mobile';
import { ArrayElement } from './utils';

const SCRAPERS = [
  //
  scrapeFBPost,
  scrapeFBPostList,
  scrapeFBMobilePost,
  scrapeFBMobilePostList,
] as const;

export type ScrapedData = ExtractScraperGeneric<ArrayElement<typeof SCRAPERS>>

async function scrape(): Promise<ScrapedData | null> {
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

  return null;
}

declare global {
  interface Window {
    originalURL: URL;
    _scrape: () => Promise<ScrapedData | null>;
    _onScrape?: (result: ScrapedData | null, error: string | null) => void;
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
