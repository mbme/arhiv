import { scrollModalToTop } from '../scripts/modal';
import { fetchAndReplace } from '../scripts/utils';

export function initPickFileModal(modalEl: HTMLElement): void {
  modalEl.querySelectorAll('a[data-component=entry]').forEach((el) => {
    if (!(el instanceof HTMLAnchorElement)) {
      throw new Error('entry must be anchor');
    }

    el.addEventListener('click', (e) => {
      e.preventDefault();


      void fetchAndReplace(el.href, modalEl);
      scrollModalToTop(modalEl);
    });
  });

  modalEl.querySelectorAll('a[data-component=toggle-hidden]').forEach((el) => {
    if (!(el instanceof HTMLAnchorElement)) {
      throw new Error('toggle must be anchor');
    }

    el.addEventListener('click', (e) => {
      e.preventDefault();


      void fetchAndReplace(el.href, modalEl);
      scrollModalToTop(modalEl);
    });
  });
}
