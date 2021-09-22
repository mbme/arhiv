function evalDataJS(el: Element): boolean {
  const script = el.getAttribute('data-js');
  if (script === null) {
    return false;
  }

  try {
    // eslint-disable-next-line @typescript-eslint/no-implied-eval
    const fn = window.Function(`"use strict"; ${script}`);

    fn.apply(el);

    el.removeAttribute('data-js');

    return true;
  } catch (e) {
    console.error('Failed to execute the "data-js" attribute: %s\n script: %s\n', e, script, el);
    return false;
  }
}

export function initDataJS(observeChanges = false): void {
  for (const el of document.querySelectorAll('[data-js]')) {
    evalDataJS(el);
  }

  if (!observeChanges) {
    return;
  }

  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      for (const node of mutation.addedNodes) {
        if (node.nodeType === Node.ELEMENT_NODE) {
          evalDataJS(node as Element);
        }
      }
    }
  });

  observer.observe(document.body, {
    subtree: true,
    childList: true,
  });
}
