import { renderNotification } from './notification';

async function writeText(text: string): Promise<void> {
  await navigator.clipboard.writeText(text);
}

function copyText(text: string): Promise<void> {
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

export async function copyTextToClipboard(text: string, textName: string): Promise<void> {
  try {
    await copyText(text);
    renderNotification(`Copied ${textName} to the clipboard!`);
  } catch (e) {
    console.error(`Failed to copy ${textName} to the clipboard`, e);
    renderNotification(`Failed to copy ${textName} to the clipboard!`, 'error');
  }
}
