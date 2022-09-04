export type Obj<T = string> = Record<string, T | undefined>;
export type EmptyObj = Obj<never>;
export type JSONValue =
  | null
  | string
  | number
  | boolean
  | { [prop: string]: JSONValue }
  | JSONValue[];
export type JSONObj = Obj<JSONValue>;

export type ArrayElement<ArrayType extends readonly unknown[]> =
  ArrayType extends readonly (infer ElementType)[] ? ElementType : never;

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

export function cx(
  ...args: Array<string | null | undefined | false | Obj<string | null | undefined | boolean>>
): string {
  const result = [];

  for (const arg of args) {
    if (!arg) {
      continue;
    }

    if (typeof arg === 'string') {
      result.push(arg);
      continue;
    }

    for (const [objKey, objVal] of Object.entries(arg)) {
      if (objVal) {
        result.push(objKey);
      }
    }
  }

  return result.join(' ');
}

export function formatDocumentType(documentType: string, subtype?: string): string {
  if (subtype) {
    return `${documentType}/${subtype}`;
  }

  return documentType;
}

export function getSessionValue<T extends JSONValue>(key: string, defaultValue: T): T {
  const value = sessionStorage.getItem(key);

  if (value === null) {
    return defaultValue;
  }

  return JSON.parse(value) as T;
}

export function setSessionValue(key: string, value: JSONValue) {
  sessionStorage.setItem(key, JSON.stringify(value));
}

const BYTES_SIZES = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
export function formatBytes(bytes: number): string {
  if (!bytes) {
    return '0';
  }

  const power = Math.floor(Math.log(bytes) / Math.log(1024));

  const value = (bytes / Math.pow(1024, power)).toFixed(2);
  return `${value} ${BYTES_SIZES[power]}`;
}

export function setElementAttribute(
  el: HTMLElement,
  attribute: string,
  value?: string | boolean | null
) {
  if (typeof value === 'string') {
    el.setAttribute(attribute, value);
    return;
  }

  if (value === true) {
    el.setAttribute(attribute, 'true');
    return;
  }

  el.removeAttribute(attribute);
}
