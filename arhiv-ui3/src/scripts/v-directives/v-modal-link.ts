import { registerVDirective } from '../v-js';
import { initDynamicLink, isAnchorElement } from '../forms';
import { getModalContainer } from '../modal';

registerVDirective('v-modal-link', (linkEl) => {
  if (!isAnchorElement(linkEl)) {
    throw new Error('must be applied to link');
  }

  const modalEl = getModalContainer(linkEl);

  initDynamicLink(linkEl, modalEl);
});
