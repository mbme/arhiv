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

        throw e;
      }
    }

    SCRAPER.results.push(result);

    return result;
  },

  injectScraperUI() {
    const button = document.createElement('button');
    button.onclick = () => window._scraper.scrape();
    button.innerText = '[ SCRAPE! ]';
    button.style.background = 'white';
    button.style.color = 'orange';
    button.style.border = '1px solid orange';
    button.style.fontSize = 'xx-large';
    button.style.cursor = 'pointer';

    button.style.position = 'fixed';
    button.style.zIndex = '1000';
    button.style.top = '5%';
    button.style.right = '10%';

    document.body.insertAdjacentElement('afterbegin', button);

    // FIXME add DONE button, wrap into some "panel"
  },

  getSelectionString,
} as const;

declare global {
  interface Window {
    originalURL: URL;
    _scraper: typeof SCRAPER;
    _doneCallback: () => void;
  }
}

window._scraper = SCRAPER;
