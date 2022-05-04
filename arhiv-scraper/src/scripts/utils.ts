import { ElementHandle, Page } from 'puppeteer-core';

type ObjValue = string | number | boolean | null | undefined;
export type Obj<TValue = ObjValue> = Record<string, TValue>;

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
  return el.$eval(selector, (node) => (node as HTMLElement).innerText);
}

export async function getListValues(
  el: ElementHandle<HTMLElement> | Page,
  selector: string
): Promise<string[]> {
  const items = await el.$$eval(selector, (nodes) =>
    nodes.map((node) => (node as HTMLElement).innerText.trim()).filter((item) => item.length > 0)
  );

  return uniqArr(items);
}

export async function getListStr(
  el: ElementHandle<HTMLElement> | Page,
  selector: string
): Promise<string> {
  const values = await getListValues(el, selector);

  return values.join(', ');
}

export function getImageSrc(
  el: ElementHandle<HTMLElement> | Page,
  selector: string
): Promise<string> {
  return el.$eval(selector, (node) => (node as HTMLImageElement).src);
}

export function getAttribute(
  el: ElementHandle<HTMLElement> | Page,
  selector: string,
  attributeName: string
): Promise<string | null> {
  return el.$eval(
    selector,
    (node, attributeName) => (node as HTMLElement).getAttribute(attributeName as string),
    attributeName
  );
}

export function removeEl(el: ElementHandle<HTMLElement> | Page, selector: string): Promise<void> {
  return el.$eval(selector, (node) => node.remove());
}

export async function getTable(
  el: ElementHandle<HTMLElement> | Page,
  rowSelector: string,
  split = ':'
): Promise<Obj<string | undefined>> {
  const items = await getListValues(el, rowSelector);

  const table = Object.fromEntries(
    items.map(
      (item) => item.split(split).map((value) => value.trim()) as [string, string | undefined]
    )
  );

  return table;
}
