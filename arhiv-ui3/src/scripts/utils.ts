export type Obj = Record<string, string | undefined>;
export type Callback = () => void;

export async function callRPCAction<T = unknown>(action: Record<string, unknown>): Promise<T> {
  try {
    const response = await fetch('/rpc', {
      method: 'POST',
      cache: 'no-cache',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(action),
    });

    if (!response.ok) {
      const message = await response.text();
      throw new Error(`action failed: ${response.status}\n${message}`);
    }

    const json = await response.json() as T;

    return json;
  } catch (e) {
    console.error(e);
    alert(e);

    throw e;
  }
}

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

// eslint-disable-next-line @typescript-eslint/no-empty-function
export const noop = (): void => {};

export function formDataToObj(fd: FormData): Obj {
  const result: Obj = {};

  for (const [key, value] of fd.entries()) {
    result[key] = value.toString();
  }

  return result;
}

export function isEqualFormData(fd1: FormData, fd2: FormData): boolean {
  const fd1Obj = formDataToObj(fd1);
  const fd2Obj = formDataToObj(fd2);

  return JSON.stringify(fd1Obj) === JSON.stringify(fd2Obj);
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

export function autoGrowTextarea(textarea: HTMLTextAreaElement): void {
  const parent = textarea.parentElement;
  if (!parent) {
    console.error("Textarea doesn't have a parent element");
    return;
  }

  const updateHeight = () => {
    // preserve height between updates
    parent.style.height = `${parent.scrollHeight}px`;

    textarea.style.height = 'auto';
    textarea.style.height = `${textarea.scrollHeight}px`;

    parent.style.height = 'auto';
  };

  updateHeight();

  textarea.addEventListener('input', updateHeight, { passive: true });
  window.addEventListener('resize', updateHeight, { passive: true });
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
