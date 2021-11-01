import {
  autoGrowTextarea,
  callRPCAction,
  formDataToObj,
  isEqualFormData,
} from './scripts/utils';
import { initDataJS } from './scripts/data-js';
import { dispatchCloseModalEvent, showModal } from './scripts/modal';
import { copyTextToClipboard } from './scripts/clipboard';

import 'unpoly';

type Document = {
  id: string,
  data: Record<string, string | undefined>,
};

class ArhivUI {
  goBack(fallback = '/') {
    if (history.length > 2) {
      history.back();
    } else {
      window.location.assign(fallback);
    }
  }

  async deleteDocument(id: string, urlOnDelete: string) {
    await callRPCAction({
      Delete: { id }
    });

    window.location.assign(urlOnDelete);
  }

  async pickAttachment() {
    const id: string = await callRPCAction({
      PickAttachment: { }
    });

    if (!id) {
      return;
    }

    console.log('Selected attachment', id);

    return copyTextToClipboard(id, 'attachment id');
  }

  async showDeleteDocumentConfirmationDialog(id: string, parentCollection = '') {
    await showModal(
      parentCollection
        ? `/modals/collections/${parentCollection}/documents/${id}/delete`
        : `/modals/documents/${id}/delete`);
  }

  async pickDocument() {
    await showModal('/modals/pick-document');
  }

  initEditorForm = (form: HTMLFormElement, originalDocument: Document, urlOnSave: string) => {
    const initialFormData = new FormData(form);

    function onBeforeUnload(event: BeforeUnloadEvent) {
      const fd = new FormData(form);

      if (isEqualFormData(initialFormData, fd)) {
        return;
      }

      event.preventDefault();

      return event.returnValue = 'Page has unsaved changes. Are you sure you want to exit?';
    }

    window.addEventListener('beforeunload', onBeforeUnload, { capture: true });

    form.addEventListener('submit', async (event: Event) => {
      event.preventDefault();

      const data = formDataToObj(new FormData(form));

      await callRPCAction({
        Save: {
          document: {
            ...originalDocument,
            data,
          },
        },
      });

      window.removeEventListener('beforeunload', onBeforeUnload, { capture: true });

      window.location.assign(urlOnSave);
    });

    form.querySelectorAll('textarea').forEach(autoGrowTextarea);
  };

  initDocumentPicker = (container: HTMLElement) => {
    container.addEventListener('click', (e) => {
      const el = (e.target as HTMLElement).closest('li');

      if (!el || !container.contains(el)) {
        return;
      }

      const id = el.dataset['id'];

      if (!id) {
        return;
      }

      console.log('Selected document', id);

      dispatchCloseModalEvent(container);

      void copyTextToClipboard(id, 'selected document id');
    });
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
