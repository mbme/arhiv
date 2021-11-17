import { submitFormAndReplace } from '../scripts/forms';
import { dispatchCloseModalEvent, scrollModalToTop } from '../scripts/modal';
import { fetchAndReplace } from '../scripts/utils';

export function initPickDocumentModal(modalEl: HTMLElement): void {
  modalEl.querySelectorAll('[data-component=catalog-entry]').forEach((el) => {
    if (!(el instanceof HTMLElement)) {
      return;
    }

    el.addEventListener('click', (e) => {
      e.preventDefault();

      const id = el.dataset.id;

      if (!id) {
        throw new Error('catalog-entry must have an id');
      }

      console.log('selected document %s', id);

      void window.arhiv_ui.copyTextToClipboard(id, 'selected document id');
      dispatchCloseModalEvent(modalEl);
    });
  });

  modalEl.querySelectorAll('a[data-component=pagination]').forEach((el) => {
    if (!(el instanceof HTMLAnchorElement)) {
      throw new Error('pagination must be anchor');
    }

    el.addEventListener('click', (e) => {
      e.preventDefault();

      void fetchAndReplace(el.href, modalEl);
      scrollModalToTop(modalEl);
    });
  });

  const searchForm = modalEl.querySelector('form[data-component=search-input]');
  if (!searchForm) {
    throw new Error('search form is missing');
  }

  if (!(searchForm instanceof HTMLFormElement)) {
    throw new Error('search form must be form');
  }

  searchForm.addEventListener('submit', (e) => {
    e.preventDefault();

    void submitFormAndReplace(searchForm, modalEl);
    scrollModalToTop(modalEl);
  });
}
