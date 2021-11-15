import { formDataToObject } from '../scripts/forms';
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

    // FIXME extract this into a helper function to submit form like browser does
    const fd = new FormData(searchForm);
    const data = formDataToObject(fd);
    const qs = new URLSearchParams(data);

    const url = new URL(searchForm.action);
    url.search = qs.toString();

    void fetchAndReplace(url.toString(), modalEl);
    scrollModalToTop(modalEl);
  });
}
