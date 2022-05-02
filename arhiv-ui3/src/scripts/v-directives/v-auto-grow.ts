import { registerVDirective } from '../v-js';

function autoGrowTextarea(textarea: HTMLTextAreaElement): void {
  const parent = textarea.parentElement;
  if (!parent) {
    console.error("Textarea doesn't have a parent element");
    return;
  }

  const updateHeight = () => {
    // preserve height between updates
    parent.style.height = `${parent.scrollHeight}px`;

    textarea.style.height = 'auto';
    textarea.style.height = `${textarea.scrollHeight}px`;

    parent.style.height = 'auto';
  };

  updateHeight();

  textarea.addEventListener('input', updateHeight, { passive: true });
  window.addEventListener('resize', updateHeight, { passive: true });
}

registerVDirective('v-auto-grow', (el) => {
  if (!(el instanceof HTMLTextAreaElement)) {
    throw new Error('v-auto-grow must be applied to textarea');
  }

  autoGrowTextarea(el);
});
