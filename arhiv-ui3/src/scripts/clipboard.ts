async function writeText(text: string): Promise<void> {
  await navigator.clipboard.writeText(text);
}

export function copyText(text: string): Promise<void> {
  if (document.hasFocus()) {
    return writeText(text);
  }

  return new Promise((resolve, reject) => {
    const handler = () => {
      writeText(text).then(resolve, reject);
    };

    window.addEventListener('focus', handler, { once: true });
  });
}
