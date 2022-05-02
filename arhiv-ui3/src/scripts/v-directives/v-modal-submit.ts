import { registerVDirective } from '../v-js';
import { initDynamicForm, isFormElement } from '../forms';
import { getModalContainer } from '../modal';

registerVDirective('v-modal-submit', (formEl) => {
  if (!isFormElement(formEl)) {
    throw new Error('must be applied to form');
  }

  const modalEl = getModalContainer(formEl);

  initDynamicForm(formEl, modalEl);
});
