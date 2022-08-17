export type Obj<T = string> = Record<string, T | undefined>;
export type EmptyObj = Obj<never>;
export type JSONValue =
  | undefined
  | null
  | string
  | number
  | boolean
  | { [prop: string]: JSONValue }
  | JSONValue[];
export type JSONObj = Obj<JSONValue>;

export type Callback = () => void;

// eslint-disable-next-line @typescript-eslint/no-empty-function
export const noop = (): void => {};

let _newIdState = 0;
export const newId = (): number => (_newIdState += 1);

export const sum = (a: number, b: number) => a + b;

export async function fetchText(
  url: string,
  body?: XMLHttpRequestBodyInit,
  headers?: HeadersInit
): Promise<string> {
  try {
    const response = await fetch(url, {
      method: body ? 'POST' : 'GET',
      headers,
      body,
    });

    const message = await response.text();

    if (!response.ok) {
      throw new Error(`failed to fetch text: ${response.status}\n${message}`);
    }

    return message;
  } catch (e) {
    console.error(e);
    alert(e);

    throw e;
  }
}

export async function fetchJSON<T>(url: string): Promise<T> {
  const result = await fetchText(url, undefined, {
    Accept: 'application/json',
  });

  return JSON.parse(result) as T;
}

export async function fetchAndReplace(url: string, el: Element, selector = ''): Promise<void> {
  const content = await fetchText(url);

  replaceEl(el, content, selector);
}

function selectNodes(content: string, selector: string): NodeListOf<Element> {
  const domEl = document.createElement('div');
  domEl.innerHTML = content;

  return domEl.querySelectorAll(selector);
}

export function replaceEl(el: Element, content: string, selector = ''): void {
  if (selector.length === 0) {
    el.outerHTML = content;
  } else {
    el.replaceWith(...selectNodes(content, selector));
  }
}

export function replaceElContent(el: Element, content: string, resetScroll = true): void {
  el.innerHTML = content;

  if (resetScroll) {
    el.scrollTop = 0;
  }
}

export function setQueryParam(urlStr: string, param: string, value: string | undefined): string {
  const url = new URL(urlStr, window.location.href);

  if (value === undefined) {
    url.searchParams.delete(param);
  } else {
    url.searchParams.set(param, value);
  }

  return url.toString();
}

export function updateQueryParam(param: string, value: string | undefined): void {
  const searchParams = new URLSearchParams(window.location.search);

  if (value) {
    searchParams.set(param, value);
  } else {
    searchParams.delete(param);
  }

  window.history.replaceState({}, '', '?' + searchParams.toString());
}

export function lockGlobalScroll(): Callback {
  const documentEl = document.documentElement;
  const originalStyle = documentEl.style.cssText;
  const scrollTop = documentEl.scrollTop;

  // preserve scroll position and hide scroll
  documentEl.style.cssText = `position: fixed; left: 0; right: 0; overflow: hidden; top: -${scrollTop}px`;

  return () => {
    // restore scroll position
    documentEl.style.cssText = originalStyle;
    documentEl.scrollTop = scrollTop;
  };
}

export function pickRandomElement<T>(arr: T[]): T {
  const pos = Math.floor(Math.random() * arr.length);

  return arr[pos];
}
