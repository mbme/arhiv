import { init_V_JS } from './scripts/v-js';
import { copyTextAndNotify } from './scripts/clipboard';
import './scripts/v-audio-player';
import './scripts/v-editor';
import './scripts/v-directives';

import { initPickDocumentModal } from './pages/pick_document_modal';
import { initPlayerApp } from './pages/player_app_page';
import { initDataEditor } from './components/document_data_editor';

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
  initDataEditor = initDataEditor;

  copyTextAndNotify = copyTextAndNotify;
}

declare global {
  interface Window {
    arhiv_ui: ArhivUI;
  }
}

window.arhiv_ui = new ArhivUI();

window.addEventListener('DOMContentLoaded', () => {
  init_V_JS(true);
});
