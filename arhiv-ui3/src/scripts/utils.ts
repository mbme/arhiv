export type Obj = Record<string, string | undefined>;

export function formDataToObj(fd: FormData): Obj {
  const result: Obj = {};

  for (const [key, value] of fd.entries()) {
    result[key] = value.toString();
  }

  return result;
}

export function isEqualFormData(fd1: FormData, fd2: FormData): boolean {
  const fd1Obj = formDataToObj(fd1);
  const fd2Obj = formDataToObj(fd2);

  return JSON.stringify(fd1Obj) === JSON.stringify(fd2Obj);
}

export function copyTextToClipboard(text: string): void {
  const writeText = () => {
    navigator.clipboard.writeText(text).catch((e) => {
      console.error('failed to copy text to clipboard', e);
    });
  };

  if (document.hasFocus()) {
    writeText();
  } else {
    window.addEventListener('focus', writeText, { once: true });
  }
}

export function replaceEl(el: HTMLElement, content: string): void {
  el.outerHTML = content;
}
