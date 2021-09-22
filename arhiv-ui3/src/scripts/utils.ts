import A11yDialog from 'a11y-dialog';

export type Obj = Record<string, string | undefined>;
type Callback = () => void;

export async function call_action(action: Record<string, unknown>): Promise<unknown> {
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
      throw new Error(`action failed: ${response.status}`);
    }

    return response.json();
  } catch (e) {
    console.error(e);
    alert(e);

    throw e;
  }
}

// eslint-disable-next-line @typescript-eslint/no-empty-function
const noop = () => {};

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

export function copyTextToClipboard(text: string): void {
  const writeText = () => {
    navigator.clipboard.writeText(text).catch((e) => {
      console.error('failed to copy text to clipboard', e);
    });
  };

  if (document.hasFocus()) {
    writeText();
  } else {
    window.addEventListener('focus', writeText, { once: true });
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

function lockGlobalScroll(): Callback {
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

export function renderModal(modal: string): void {
  const rootEl = document.getElementById('modal-root');
  if (!rootEl) {
    throw new Error('modal root el not found');
  }

  const domEl = document.createElement('div');
  domEl.innerHTML = modal;

  const modalEl = domEl.firstElementChild;
  if (!modalEl) {
    throw new Error("modal content doesn't have a child");
  }

  rootEl.appendChild(modalEl);

  const dialog = new A11yDialog(modalEl);

  let unlockScroll = noop;
  dialog.on('show', () => {
    unlockScroll = lockGlobalScroll();
  });

  dialog.on('hide', () => {
    unlockScroll();

    modalEl.remove();
  });

  dialog.show();
}
