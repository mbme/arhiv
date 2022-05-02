import { registerVDirective } from '../v-js';

registerVDirective('v-js', (el, value) => {
  // eslint-disable-next-line @typescript-eslint/no-implied-eval
  const fn = window.Function(`"use strict"; ${value}`);

  fn.apply(el);
});
