import { copyTextAndNotify } from '../scripts/clipboard';
import { isFormElement, initDynamicLink, initDynamicForm, isAnchorElement } from '../scripts/forms';
import { dispatchCloseModalEvent } from '../scripts/modal';
import { replaceElContent } from '../scripts/utils';

export function initPickDocumentModal(modalEl: HTMLElement): void {
  modalEl.querySelectorAll('[data-component=catalog-entry]').forEach((el) => {
    if (!(el instanceof HTMLElement)) {
      throw new Error('must be applied to HTML element');
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

  modalEl.querySelectorAll('a[data-component=pagination]').forEach((el) => {
    if (!isAnchorElement(el)) {
      throw new Error('must be applied to link');
    }

    initDynamicLink(el, modalEl);
  });

  const searchForm = modalEl.querySelector('form[data-component=search-input]');
  if (!isFormElement(searchForm)) {
    throw new Error('search form must be present');
  }

  initDynamicForm(searchForm, (content) => replaceElContent(modalEl, content));
}
