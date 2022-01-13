import { ElementHandle, Page } from 'puppeteer-core';

export type Obj = Record<string, string | number | boolean | null | undefined>;

export function uniqArr<T>(arr: T[]): T[] {
  const result: T[] = [];

  for (const item of arr) {
    if (!result.includes(item)) {
      result.push(item);
    }
  }

  return result;
}

export function getText(el: ElementHandle<HTMLElement> | Page, selector: string): Promise<string> {
  return el.$eval(selector, node => (node as HTMLElement).innerText);
}

export async function getListValues(el: ElementHandle<HTMLElement> | Page, selector: string): Promise<string[]> {
  const items = await el.$$eval(selector, nodes => nodes.map(node => (node as HTMLElement).innerText.trim()).filter(item => item.length > 0));

  return uniqArr(items);
}

export async function getListStr(el: ElementHandle<HTMLElement> | Page, selector: string): Promise<string> {
  const values = await getListValues(el, selector);

  return values.join(', ');
}

export function getImageSrc(el: ElementHandle<HTMLElement> | Page, selector: string): Promise<string> {
  return  el.$eval(selector, node => (node as HTMLImageElement).src);
}

export function removeEl(el: ElementHandle<HTMLElement> | Page, selector: string): Promise<void> {
  return el.$eval(selector, node => node.remove());
}
