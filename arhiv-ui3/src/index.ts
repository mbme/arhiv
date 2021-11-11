import 'unpoly';

import {
  autoGrowTextarea,
  isEqualFormData,
  keepSessionState,
} from './scripts/utils';
import { init_V_JS } from './scripts/v-js';
import { renderNotification } from './scripts/notification';
import { copyText } from './scripts/clipboard';

class ArhivUI {
  goBack(fallback = '/') {
    if (history.length > 2) {
      history.back();
    } else {
      window.location.assign(fallback);
    }
  }

  async pickAttachment() {
    const response = await fetch('/rpc/pick-attachment', {
      method: 'POST',
      cache: 'no-cache',
    });
    const message = await response.text();

    if (!response.ok) {
      throw new Error(`failed to pick attachment: ${response.status}\n${message}`);
    }

    if (!message) {
      return;
    }

    console.log('Selected attachment', message);

    return this.copyTextToClipboard(message, 'attachment id');
  }

  preserveUnsavedChanges = (form: HTMLFormElement) => {
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
  };

  autoGrowTextarea = autoGrowTextarea;

  keepSessionState = keepSessionState;

  copyTextToClipboard = async (text: string, textName: string): Promise<void> => {
    try {
      await copyText(text);
      renderNotification(`Copied ${textName} to clipboard!`);
    } catch (e) {
      console.error(`Failed to copy ${textName} to clipboard`, e);
      renderNotification(`Failed to copy ${textName} to clipboard!`, 'error');
    }
  };
}

declare global {
  interface Window {
    arhiv_ui: ArhivUI
  }
}

history.scrollRestoration = 'auto';

window.arhiv_ui = new ArhivUI();

window.addEventListener('DOMContentLoaded', () => {
  init_V_JS(true);
});
