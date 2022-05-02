import { registerVDirective } from '../v-js';
import { dispatchCloseModalEvent } from '../modal';

registerVDirective('v-modal-close', (el) => {
  el.addEventListener('click', () => dispatchCloseModalEvent(el));
});
