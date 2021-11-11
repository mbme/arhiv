export type Obj<T = string> = Record<string, T | undefined>;
export type Callback = () => void;

// eslint-disable-next-line @typescript-eslint/no-empty-function
export const noop = (): void => {};

export const sum = (a: number, b: number) => a + b;

export async function fetchHTML(url: string): Promise<string> {
  try {
    const response = await fetch(url);

    const message = await response.text();

    if (!response.ok) {
      throw new Error(`failed to fetch HTML: ${response.status}\n${message}`);
    }

    return message;
  } catch (e) {
    console.error(e);
    alert(e);

    throw e;
  }
}

function selectNodes(content: string, selector: string): NodeListOf<Element> {
  const domEl = document.createElement('div');
  domEl.innerHTML = content;

  return domEl.querySelectorAll(selector);
}

export function replaceEl(el: HTMLElement, content: string, selector: string): void {
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
