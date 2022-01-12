import { default as puppeteer, Browser, Page } from 'puppeteer-core';
import { ActionChannel } from './ActionChannel';

export class Context {
  readonly channel = new ActionChannel();

  private _browser?: Browser;

  constructor(
    private _chromeBinPath: string,
    private _debug: boolean,
  ) {}

  private async getBrowser(): Promise<Browser> {
    if (this._browser) {
      return this._browser;
    }

    const executablePath = this._chromeBinPath;

    this._browser = await puppeteer.launch(this._debug ? {
      executablePath,
      headless: false,
      defaultViewport: null, // use full viewport
    } : {
      executablePath,
    });

    return this._browser;
  }

  async newPage(url: string): Promise<Page> {
    const browser = await this.getBrowser();

    const page = await browser.newPage();
    await page.goto(url);

    return page;
  }

  async close(): Promise<void> {
    this.channel.close();
    await this._browser?.close();
  }
}

export function uniqArr<T>(arr: T[]): T[] {
  const result: T[] = [];

  for (const item of arr) {
    if (!result.includes(item)) {
      result.push(item);
    }
  }

  return result;
}
