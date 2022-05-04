import { registerVDirective } from '../v-js';
import { initDynamicForm, isFormElement } from '../forms';

type Options = {
  onSuccess?: string;
};

registerVDirective('v-modal-submit', (formEl, arg) => {
  if (!isFormElement(formEl)) {
    throw new Error('must be applied to form');
  }

  const options = JSON.parse(arg || '{}') as Options;

  initDynamicForm(formEl, () => {
    if (options.onSuccess) {
      window.location.href = options.onSuccess;
    }
  });
});
