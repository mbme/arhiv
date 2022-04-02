export type Obj<T = string> = Record<string, T | undefined>;
export type Callback = () => void;

// eslint-disable-next-line @typescript-eslint/no-empty-function
export const noop = (): void => {};

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

export function keepSessionState(el: HTMLElement, key: string): void {
  const sessionKey = `state-${el.tagName}-${key}-${location.pathname}`;
  const savedState = sessionStorage.getItem(sessionKey);

  if (el instanceof HTMLDetailsElement) {
    if (savedState) {
      el.open = savedState === 'true';
    }

    el.addEventListener('toggle', () => sessionStorage.setItem(sessionKey, el.open.toString()));

    return;
  }

  throw new Error(`unsupported element "${el.tagName}"`);
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
