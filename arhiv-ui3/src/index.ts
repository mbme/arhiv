import { keepSessionState } from './scripts/utils';
import { init_V_JS } from './scripts/v-js';
import { renderNotification } from './scripts/notification';
import { copyText } from './scripts/clipboard';
import { autoGrowTextarea, preserveUnsavedChanges } from './scripts/forms';
import { dispatchCloseModalEvent, showModal } from './scripts/modal';

import { initPickDocumentModal } from './pages/pick_document_modal';
import { initPickFileModal } from './pages/pick_file_modal';
import { initPickFileConfirmationModal } from './pages/pick_file_confirmation_modal';

class ArhivUI {
  goBack(fallback = '/') {
    if (history.length > 2) {
      history.back();
    } else {
      window.location.assign(fallback);
    }
  }

  initPickDocumentModal = initPickDocumentModal;
  initPickFileModal = initPickFileModal;
  initPickFileConfirmationModal = initPickFileConfirmationModal;

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

window.arhiv_ui = new ArhivUI();

window.addEventListener('DOMContentLoaded', () => {
  init_V_JS(true, {
    'v-js': (el, value) => {
      // eslint-disable-next-line @typescript-eslint/no-implied-eval
      const fn = window.Function(`"use strict"; ${value}`);

      fn.apply(el);
    },
    'v-auto-grow': (el) => {
      if (!(el instanceof HTMLTextAreaElement)) {
        throw new Error('v-auto-grow must be applied to textarea');
      }

      autoGrowTextarea(el);
    },
    'v-preserve-unsaved-changes': (el) => {
      if (!(el instanceof HTMLFormElement)) {
        throw new Error('v-preserve-unsaved-changes must be applied to form');
      }

      preserveUnsavedChanges(el);
    },
    'v-keep-state': (el, value) => {
      if (!(el instanceof HTMLElement)) {
        throw new Error('v-keep-state must be applied to html elements');
      }

      keepSessionState(el, value);
    },

    'v-layer': (el, value) => {
      if (!(el instanceof HTMLButtonElement)) {
        throw new Error('v-layer must be applied to button');
      }

      el.addEventListener('click', () => void showModal(value));
    },

    'v-modal-close': (el) => {
      el.addEventListener('click', () => dispatchCloseModalEvent(el));
    },
  });
});
