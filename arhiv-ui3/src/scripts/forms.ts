import { fetchText, replaceEl } from './utils';

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

    return (event.returnValue = 'Page has unsaved changes. Are you sure you want to exit?');
  }

  window.addEventListener('beforeunload', onBeforeUnload, { capture: true });

  form.addEventListener('submit', () => {
    submitted = true;
  });
}

export function submitForm(form: HTMLFormElement): Promise<string> {
  if (form.enctype !== 'application/x-www-form-urlencoded') {
    throw new Error('unimplemented: only application/x-www-form-urlencoded supported yet');
  }

  const fd = new FormData(form);
  const data = formDataToObject(fd);

  const qs = new URLSearchParams(data);

  if (form.method === 'get') {
    const url = new URL(form.action);
    url.search = qs.toString();

    return fetchText(url.toString());
  }

  // send URLSearchParams instance so that body is x-www-form-urlencoded
  return fetchText(form.action, qs);
}

export async function submitFormAndReplace(
  form: HTMLFormElement,
  targetEl: HTMLElement
): Promise<void> {
  const content = await submitForm(form);

  replaceEl(targetEl, content);
}

export function isFormElement(el: Element | null): el is HTMLFormElement {
  return el instanceof HTMLFormElement;
}

export function initDynamicForm(
  formEl: HTMLFormElement,
  onSuccess: (content: string) => void
): void {
  let inProgress = false;

  formEl.addEventListener('submit', (e) => {
    e.preventDefault();

    if (inProgress) {
      console.warn('Submit already in progress, ignoring');
      return;
    }

    inProgress = true;

    submitForm(formEl)
      .then(
        (content) => {
          onSuccess(content);
        },
        (err) => {
          console.error('Failed to submit form', err);
        }
      )
      .finally(() => {
        inProgress = false;
      });
  });
}

export function isAnchorElement(el: Element | null): el is HTMLAnchorElement {
  return el instanceof HTMLAnchorElement;
}

export function initDynamicLink(linkEl: HTMLAnchorElement, containerEl: HTMLElement): void {
  linkEl.addEventListener('click', (e) => {
    e.preventDefault();

    void fetchText(linkEl.href).then((content) => {
      containerEl.innerHTML = content;
      containerEl.scrollTop = 0;
    });
  });
}
