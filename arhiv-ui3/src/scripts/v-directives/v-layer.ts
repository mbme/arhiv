import { registerVDirective } from '../v-js';
import { showModal } from '../modal';

registerVDirective('v-layer', (el, value) => {
  if (!(el instanceof HTMLButtonElement)) {
    throw new Error('v-layer must be applied to button');
  }

  el.addEventListener('click', () => void showModal(value));
});
