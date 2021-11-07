import 'unpoly';

import {
  autoGrowTextarea,
  callRPCAction,
  isEqualFormData,
} from './scripts/utils';
import { initDataJS } from './scripts/data-js';
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
    const id: string = await callRPCAction({
      PickAttachment: { }
    });

    if (!id) {
      return;
    }

    console.log('Selected attachment', id);

    return this.copyTextToClipboard(id, 'attachment id');
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

  copyTextToClipboard = async (text: string, textName: string): Promise<void> => {
    try {
      await copyText(text);
      renderNotification(`Copied ${textName} to the clipboard!`);
    } catch (e) {
      console.error(`Failed to copy ${textName} to the clipboard`, e);
      renderNotification(`Failed to copy ${textName} to the clipboard!`, 'error');
    }
  };
}

declare global {
  interface Window {
    arhiv_ui: ArhivUI
  }
}

window.arhiv_ui = new ArhivUI();

window.addEventListener('DOMContentLoaded', () => {
  initDataJS(true);
});
