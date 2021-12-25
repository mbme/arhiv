import { Browser } from 'puppeteer-core';
import { ActionChannel } from './ActionChannel';

export type Obj = Record<string, string | number | boolean | null | undefined>;

export type Importer = (url: string, browser: Browser, channel: ActionChannel) => Promise<boolean>;

export function uniqArr<T>(arr: T[]): T[] {
  const result: T[] = [];

  for (const item of arr) {
    if (!result.includes(item)) {
      result.push(item);
    }
  }

  return result;
}
