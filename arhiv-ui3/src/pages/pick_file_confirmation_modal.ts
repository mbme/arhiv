import { submitFormAndReplace } from '../scripts/forms';
import { scrollModalToTop } from '../scripts/modal';

export function initPickFileConfirmationModal(modalEl: HTMLElement): void {
  const confirmationForm = modalEl.querySelector('form[data-component=confirmation]');
  if (!confirmationForm) {
    throw new Error('confirmation form must be present');
  }

  if (!(confirmationForm instanceof HTMLFormElement)) {
    throw new Error('unreachable: must be form');
  }

  confirmationForm.addEventListener('submit', (e) => {
    e.preventDefault();


    void submitFormAndReplace(confirmationForm, modalEl);
    scrollModalToTop(modalEl);
  });
}
