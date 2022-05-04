import { registerVDirective } from '../v-js';
import { preserveUnsavedChanges } from '../forms';

registerVDirective('v-preserve-unsaved-changes', (el) => {
  if (!(el instanceof HTMLFormElement)) {
    throw new Error('v-preserve-unsaved-changes must be applied to form');
  }

  preserveUnsavedChanges(el);
});
