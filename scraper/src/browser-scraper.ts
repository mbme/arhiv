import { ScrapedData, SCRAPERS } from './scrapers';
import { getSelectionString } from './utils';

export type ScrapeResult = {
  url: string;
  originalUrl?: string;

  scraperName?: string;

  data?: ScrapedData;
  error?: string;
};

const scraper = {
  async scrape(): Promise<ScrapeResult> {
    const result: ScrapeResult = {
      url: location.href,
      originalUrl: window.originalURL?.toString(),
    };

    const locationURL = new URL(window.location.href);

    for (const scraper of SCRAPERS) {
      try {
        const data = await scraper(locationURL);

        if (data) {
          console.info(`scraper ${scraper.name} succeeded`);

          result.scraperName = scraper.name;
          result.data = data;

          break;
        }
      } catch (e) {
        console.error(`scraper ${scraper.name} failed:`, e);

        result.scraperName = scraper.name;
        result.error = (e as Error).toString();

        window._onScrape?.(result);

        throw e;
      }
    }

    window._onScrape?.(result);

    return result;
  },
  getSelectionString,
} as const;

declare global {
  interface Window {
    originalURL: URL;
    _scraper: typeof scraper;
    _onScrape?: (result: ScrapeResult) => void;
  }
}

window._scraper = scraper;
