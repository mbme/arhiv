import { Obj } from './utils';

function formDataToObj(fd: FormData): Obj {
  const result: Obj = {};

  for (const [key, value] of fd.entries()) {
    result[key] = value.toString();
  }

  return result;
}

function isEqualFormData(fd1: FormData, fd2: FormData): boolean {
  const fd1Obj = formDataToObj(fd1);
  const fd2Obj = formDataToObj(fd2);

  return JSON.stringify(fd1Obj) === JSON.stringify(fd2Obj);
}

export function autoGrowTextarea(textarea: HTMLTextAreaElement): void {
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

export function preserveUnsavedChanges(form: HTMLFormElement): void {
  let submitted = false;
  const initialFormData = new FormData(form);

  function onBeforeUnload(event: BeforeUnloadEvent) {
    const fd = new FormData(form);

    if (submitted) {
      return;
    }

    if (isEqualFormData(initialFormData, fd)) {
      return;
    }

    event.preventDefault();

    return event.returnValue = 'Page has unsaved changes. Are you sure you want to exit?';
  }

  window.addEventListener('beforeunload', onBeforeUnload, { capture: true });

  form.addEventListener('submit', () => {
    submitted = true;
  });
}
