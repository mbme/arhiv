import { fetchHTML, replaceEl } from './utils';

export function formDataToObject(fd: FormData): Record<string, string> {
  const result: Record<string, string> = {};

  for (const [key, value] of fd.entries()) {
    if (value instanceof File) {
      throw new Error('unsupported: FormData contains a File');
    }

    if (result.hasOwnProperty(key)) {
      throw new Error(`unsupported: FormData contains duplicate key "${key}"`);
    }

    result[key] = value;
  }

  return result;
}

function isEqualFormData(fd1: FormData, fd2: FormData): boolean {
  const fd1Obj = formDataToObject(fd1);
  const fd2Obj = formDataToObject(fd2);

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

export function submitForm(form: HTMLFormElement): Promise<string> {
  const fd = new FormData(form);
  const data = formDataToObject(fd);

  if (form.method !== 'get') {
    throw new Error('unimplemented: only GET method supported yet');
  }

  const qs = new URLSearchParams(data);

  const url = new URL(form.action);
  url.search = qs.toString();

  return fetchHTML(url.toString());
}

export async function submitFormAndReplace(form: HTMLFormElement, targetEl: HTMLElement): Promise<void> {
  const content = await submitForm(form);

  replaceEl(targetEl, content);
}
