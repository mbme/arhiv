import { Browser } from 'puppeteer-core';

export type Obj = Record<string, string | undefined>;

export type Importer = (url: string, browser: Browser) => Promise<Obj | undefined>;

export function uniqArr<T>(arr: Array<T>) {
  const result: T[] = [];

  for (const item of arr) {
    if (!result.includes(item)) {
      result.push(item);
    }
  }

  return result;
}
