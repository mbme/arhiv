const processedElements = new WeakSet<Element>();

function evalDataJS(el: Element): boolean {
  const script = el.getAttribute('data-js');
  if (script === null) {
    return false;
  }

  if (processedElements.has(el)) {
    return false;
  }

  try {
    // eslint-disable-next-line @typescript-eslint/no-implied-eval
    const fn = window.Function(`"use strict"; ${script}`);

    fn.apply(el);

    processedElements.add(el);

    return true;
  } catch (e) {
    console.error('Failed to execute the "data-js" attribute: %s\n script: %s\n', e, script, el);
    return false;
  }
}

function processElements(): number {
  let nodesProcessed = 0;

  for (const el of document.querySelectorAll('[data-js]')) {
    nodesProcessed += evalDataJS(el) ? 1 : 0;
  }

  return nodesProcessed;
}

export function initDataJS(observeChanges = false): void {
  {
    const nodesProcessed = processElements();
    console.debug('[data-js]: processed %s nodes on init', nodesProcessed);
  }

  if (!observeChanges) {
    return;
  }

  const observer = new MutationObserver((mutations) => {
    const hasNewNodes = mutations.find(mutation => mutation.addedNodes.length > 0);

    if (!hasNewNodes) {
      return;
    }

    const nodesProcessed = processElements();

    console.debug('[data-js]: processed %s nodes on dom mutation', nodesProcessed);
  });

  observer.observe(document.body, {
    subtree: true,
    childList: true,
  });
}
