import { ScrapedData, SCRAPERS } from './scrapers';
import { getSelectionString } from './utils';

export type ScrapeResult = {
  url: string;
  originalUrl?: string;

  scraperName?: string;

  data?: ScrapedData;
  error?: string;
};

const SCRAPER = {
  results: [] as ScrapeResult[],

  async scrape(): Promise<ScrapeResult> {
    const result: ScrapeResult = {
      url: location.href,
      originalUrl: window.originalURL?.toString(),
    };

    for (const scraper of SCRAPERS) {
      if (!scraper.pattern.test(window.location.href)) {
        continue;
      }

      try {
        const data = await scraper.scrape();
        console.info(`scraper ${scraper.constructor.name} succeeded`);

        result.scraperName = scraper.constructor.name;
        result.data = data;

        break;
      } catch (e) {
        console.error(`scraper ${scraper.constructor.name} failed:`, e);

        result.scraperName = scraper.constructor.name;
        result.error = (e as Error).toString();

        SCRAPER.results.push(result);
        window._onScrape?.(result);

        throw e;
      }
    }

    SCRAPER.results.push(result);
    window._onScrape?.(result);

    return result;
  },

  getSelectionString,
} as const;

declare global {
  interface Window {
    originalURL: URL;
    _scraper: typeof SCRAPER;
    _onScrape?: (result: ScrapeResult) => void;
  }
}

window._scraper = SCRAPER;
