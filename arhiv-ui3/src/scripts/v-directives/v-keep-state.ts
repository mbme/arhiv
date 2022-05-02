import { registerVDirective } from '../v-js';

function keepSessionState(el: HTMLElement, key: string): void {
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

registerVDirective('v-keep-state', (el, value) => {
  if (!(el instanceof HTMLElement)) {
    throw new Error('v-keep-state must be applied to html elements');
  }

  keepSessionState(el, value);
});
