import { keepSessionState } from './scripts/utils';
import { init_V_JS } from './scripts/v-js';
import { copyTextAndNotify } from './scripts/clipboard';
import {
  autoGrowTextarea,
  isFormElement,
  initDynamicLink,
  initDynamicForm,
  preserveUnsavedChanges,
  isAnchorElement,
} from './scripts/forms';
import { dispatchCloseModalEvent, getModalContainer, showModal } from './scripts/modal';

import { initPickDocumentModal } from './pages/pick_document_modal';
import { initPlayerApp } from './pages/player_app_page';
import './scripts/v-audio-player';
import './scripts/v-editor';

class ArhivUI {
  goBack(fallback = '/') {
    if (history.length > 2) {
      history.back();
    } else {
      window.location.assign(fallback);
    }
  }

  initPickDocumentModal = initPickDocumentModal;

  initPlayerApp = initPlayerApp;

  copyTextAndNotify = copyTextAndNotify;
}

declare global {
  interface Window {
    arhiv_ui: ArhivUI;
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

    'v-modal-submit': (formEl) => {
      if (!isFormElement(formEl)) {
        throw new Error('must be applied to form');
      }

      const modalEl = getModalContainer(formEl);

      initDynamicForm(formEl, modalEl);
    },

    'v-modal-link': (linkEl) => {
      if (!isAnchorElement(linkEl)) {
        throw new Error('must be applied to link');
      }

      const modalEl = getModalContainer(linkEl);

      initDynamicLink(linkEl, modalEl);
    },
  });
});
