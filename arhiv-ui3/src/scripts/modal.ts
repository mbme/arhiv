import A11yDialog from 'a11y-dialog';
import { Callback, fetchHTML, noop } from './utils';

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

const CLOSE_MODAL_EVENT = 'close-modal';

function renderModal(modal: string): void {
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

  rootEl.addEventListener(CLOSE_MODAL_EVENT, () => {
    dialog.hide();
  });

  dialog.show();
}

export async function showModal(url: string): Promise<void> {
  const content = await fetchHTML(url);

  renderModal(content);
}

export function dispatchCloseModalEvent(modalChild: HTMLElement): void {
  const event = new CustomEvent(CLOSE_MODAL_EVENT, {
    bubbles: true,
  });

  modalChild.dispatchEvent(event);
}
