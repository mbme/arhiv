import * as chrono from 'chrono-node';

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

export const waitForTimeout = (timeoutMs: number) =>
  new Promise<void>((resolve) => setTimeout(resolve, timeoutMs));

export const waitForFunction = async (
  callback: () => boolean,
  description: string,
  timeoutMs = 30000,
): Promise<void> => {
  if (timeoutMs === 0) {
    throw new Error('timeoutMs must be positive number');
  }

  const ATTEMPT_TIMEOUT_MS = 500;
  const maxAttempts = Math.ceil(timeoutMs / ATTEMPT_TIMEOUT_MS);

  let attempt = 1;

  while (attempt <= maxAttempts) {
    if (callback()) {
      return;
    }

    if (attempt < maxAttempts) {
      await waitForTimeout(ATTEMPT_TIMEOUT_MS);
    }

    attempt += 1;
  }

  throw new Error(`waitForFunction: "${description}" timed out`);
};

export const waitForSelector = async <T extends Element = HTMLElement>(
  el: HTMLElement | Document,
  selector: string,
  description: string,
  timeoutMs = 30000,
): Promise<T> => {
  await waitForFunction(() => !!el.querySelector(selector), description, timeoutMs);

  return getEl<T>(selector, description);
};

export const parseHumanDate = (dateStr: string): Date | undefined => {
  const date: Date | null = chrono.parseDate(dateStr);

  return date ?? undefined;
};

type GetEl = {
  <T extends Element = HTMLElement>(selector: string): T | null;
  <T extends Element = HTMLElement>(selector: string, description: string): T;
  <T extends Element = HTMLElement>(root: HTMLElement, selector: string): T | null;
  <T extends Element = HTMLElement>(root: HTMLElement, selector: string, description: string): T;
};

export const getEl: GetEl = <T extends Element = HTMLElement>(
  arg1: string | HTMLElement | Document,
  arg2?: string,
  arg3?: string,
): T | null => {
  let selector: string;
  let root: HTMLElement | Document = document;
  let description: string | undefined = undefined;

  if (typeof arg1 === 'string') {
    selector = arg1;
    description = arg2;
  } else {
    root = arg1;

    if (typeof arg2 !== 'string') {
      throw new Error('expected second argument to be a string');
    }
    selector = arg2;

    description = arg3;
  }
  const el = root.querySelector(selector);

  if (!el) {
    if (description !== undefined) {
      throw new Error(`can't find '${description}' using selector '${selector}'`);
    }

    return el;
  }

  return el as T;
};

export const getAll = <T extends Element = HTMLElement>(
  root: HTMLElement | Document,
  selector: string,
): T[] => {
  const results = Array.from(root.querySelectorAll(selector));

  return results as T[];
};

export type ArrayElement<ArrayType extends readonly unknown[]> =
  ArrayType extends readonly (infer ElementType)[] ? ElementType : never;

export const getSelectionString = (el: HTMLElement): string => {
  const selection = window.getSelection();
  if (!selection) {
    throw new Error('selection is missing');
  }

  const range = document.createRange();
  range.selectNodeContents(el);
  selection.removeAllRanges();
  selection.addRange(range);

  const selectionStr = selection.toString();

  selection.removeAllRanges();

  return selectionStr;
};

export function getListValues(el: HTMLElement | Document | undefined, selector: string): string[] {
  const items = Array.from(el?.querySelectorAll<HTMLElement>(selector) ?? [])
    .map((el) => el.innerText.trim())
    .filter((item) => item.length > 0);

  return uniqArr(items);
}

export function getTable(
  el: HTMLElement | Document,
  rowSelector: string,
  split = ':',
): Obj<string | undefined> {
  const items = getListValues(el, rowSelector);

  const table = Object.fromEntries(
    items.map(
      (item) => item.split(split).map((value) => value.trim()) as [string, string | undefined],
    ),
  );

  return table;
}

export const getListStr = (el: HTMLElement | Document, selector: string): string => {
  const values = getListValues(el, selector);

  return values.join(', ');
};

export function scrollToBottom() {
  const scrollingElement = document.scrollingElement || document.body;
  scrollingElement.scrollTop = scrollingElement.scrollHeight;
}
