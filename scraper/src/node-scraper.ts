import { Browser, devices, Page, launch, Target } from 'puppeteer-core';
import type { ScrapeResult } from './browser-scraper';

declare const __BROWSER_SCRAPER__: string; // injected by esbuild
async function injectBrowserScraper(url: string, page: Page) {
  await page.evaluate((originalURL) => {
    window.originalURL = new URL(originalURL);
  }, url);

  await page.addScriptTag({ content: __BROWSER_SCRAPER__ });
}

async function injectScrapeButton(page: Page) {
  await page.evaluate(() => {
    const button = document.createElement('button');
    button.onclick = () => window._scraper.scrape();
    button.innerText = '[ SCRAPE! ]';
    button.style.color = 'orange';
    button.style.fontSize = 'xx-large';
    button.style.cursor = 'pointer';

    button.style.position = 'fixed';
    button.style.zIndex = '1000';
    button.style.top = '5%';
    button.style.right = '10%';

    document.body.insertAdjacentElement('afterbegin', button);
  });
}

async function openURL(page: Page, url: string, mobile?: boolean, manual?: boolean) {
  if (mobile) {
    await page.emulate(devices['iPad Pro landscape']);
  }

  await page.goto(url, {
    waitUntil: 'networkidle2',
  });

  await injectBrowserScraper(url, page);
  if (manual) {
    await injectScrapeButton(page);
  }
}

async function waitForPageClosed(browser: Browser, page: Page) {
  await new Promise<void>((resolve) => {
    browser.on('targetdestroyed', (target: Target) => {
      void target.page().then((targetPage) => {
        if (targetPage === page) {
          resolve();
        }
      });
    });
  });
}

type Options = {
  debug?: boolean;
  manual?: boolean;
  mobile?: boolean;
  executablePath?: string;
};

export async function runScrapers(url: string, options?: Options): Promise<ScrapeResult[]> {
  const browser: Browser = await launch({
    executablePath: options?.executablePath || '/usr/bin/chromium',

    headless: !options?.debug && !options?.manual,
    devtools: options?.debug,

    defaultViewport: null, // use full viewport

    args: ['--incognito'],
  });

  const page = (await browser.pages())[0];

  try {
    await openURL(page, url, options?.mobile, options?.manual);

    const results: ScrapeResult[] = [];
    await page.exposeFunction('_onScrape', (result: ScrapeResult) => {
      results.push(result);
    });

    const pageClosedPromise = waitForPageClosed(browser, page);

    if (!options?.manual) {
      await page.evaluate(() => window._scraper.scrape());
      await page.close();
    }

    await pageClosedPromise;

    return results;
  } catch (e) {
    console.error(`Failed to scrape ${url}`, e);

    if (options?.debug) {
      await page.screenshot({
        path: './failure-screenshot.png',
        fullPage: true,
      });
    }

    throw e;
  } finally {
    await browser.close();
  }
}

// if executed as a Node.js script
if (require.main === module) {
  const args = process.argv.slice(2);

  const url = args[0];
  const debug = args.indexOf('--debug') > -1;
  const mobile = args.indexOf('--mobile') > -1;
  const manual = args.indexOf('--manual') > -1;

  if (!url) {
    throw new Error('url must be specified');
  }

  const handle = (signal: string) => {
    console.error(`\nReceived ${signal}`);
  };

  process.on('SIGINT', handle);
  process.on('SIGTERM', handle);

  runScrapers(url, {
    executablePath: process.env.CHROME_BIN_PATH,
    debug,
    manual,
    mobile,
  }).then(
    (results) => {
      process.stdout.write(JSON.stringify(results, null, 2));
    },
    (e) => {
      console.error('Failed to run: ', e);
      process.exit(1);
    }
  );
}
