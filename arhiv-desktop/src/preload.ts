import { ipcRenderer } from 'electron';
import type { Action } from './index';

window.addEventListener('DOMContentLoaded', () => {
  ipcRenderer.on('action', (_event, action: Action) => {
    console.log('Action from main process:', action);

    switch (action.type) {
      case 'search': {
        // A.search(action.query);
        break;
      }
      case 'open': {
        if (action.documentId) {
          // A.open(action.documentId);
        }
        break;
      }
    }
  });
});
