import { ScrapedData, SCRAPERS } from './scrapers';
import scraperUIstr from './scraper-ui.html';

export type ScrapeResult = {
  url: string;
  originalUrl?: string;

  scraperName?: string;

  data?: ScrapedData;
  error?: string;
};

class BrowserScraper {
  public results: ScrapeResult[] = [];

  injectScraperUI() {
    document.body.insertAdjacentHTML('afterbegin', scraperUIstr);

    document
      .getElementById('_scraper-ui-btn-scrape')!
      .addEventListener('click', () => void this.scrape());

    document
      .getElementById('_scraper-ui-btn-done')!
      .addEventListener('click', () => window._doneCallback());
  }

  async scrape(): Promise<ScrapeResult> {
    const result: ScrapeResult = {
      url: location.href,
      originalUrl: window.originalURL.toString(),
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
        result.error = (e as Error).stack?.toString() ?? '';

        this._addResult(result);

        throw e;
      }
    }

    this._addResult(result);

    return result;
  }

  private _addResult(result: ScrapeResult) {
    this.results.push(result);

    const counterEl = document.querySelector('#_scraper-results-counter span');

    if (counterEl) {
      counterEl.innerHTML = this.results.length.toString();
    }
  }
}

declare global {
  interface Window {
    originalURL: URL;
    _scraper: BrowserScraper;
    _doneCallback: () => void;
  }
}

window._scraper = new BrowserScraper();
