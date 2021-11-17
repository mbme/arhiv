import { copyTextAndNotify } from '../scripts/clipboard';
import { modalLink, modalSubmit } from '../scripts/forms';
import { dispatchCloseModalEvent } from '../scripts/modal';

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

      void copyTextAndNotify(id, 'selected document id');
      dispatchCloseModalEvent(modalEl);
    });
  });

  modalEl.querySelectorAll('a[data-component=pagination]').forEach(modalLink);

  const searchForm = modalEl.querySelector('form[data-component=search-input]');
  if (!searchForm) {
    throw new Error('search form is missing');
  }

  modalSubmit(searchForm);
}
