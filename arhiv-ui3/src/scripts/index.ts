import {
  autoGrowTextarea,
  callRPCAction,
  formDataToObj,
  isEqualFormData,
  Obj,
  replaceEl,
  updateQueryParam,
} from './utils';
import { initDataJS } from './data-js';
import { dispatchCloseModalEvent, renderModal } from './modal';
import { copyTextToClipboard } from './clipboard';

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
    const dialog: string = await callRPCAction({
      RenderDeleteDocumentConfirmationDialog: {
        id,
        parent_collection: parentCollection || undefined,
      },
    });

    renderModal(dialog);
  }

  async pickDocument() {
    const dialog: string = await callRPCAction({
      RenderPickDocumentModal: {},
    });

    renderModal(dialog);
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
  }

  initCatalogLoadMore = (button: HTMLButtonElement, filter: Obj, pickerMode: boolean) => {
    button.addEventListener('click', async () => {
      const catalog: string = await callRPCAction({
        RenderCatalog: {
          filter,
          picker_mode: pickerMode,
        },
      });

      if (!button.parentElement) {
        throw new Error("button doesn't have a parent");
      }

      replaceEl(button.parentElement, catalog, 'ul > li');
    });
  }

  initCatalogSearch = (input: HTMLInputElement, documentType = '', parentCollection = '', pickerMode: boolean, queryParam = '') => {
    input.addEventListener('change', async () => {
      const pattern = input.value;

      if (queryParam) {
        updateQueryParam('pattern', pattern);
      }

      const catalog: string = await callRPCAction({
        SearchCatalog: {
          parent_collection: parentCollection || undefined,
          document_type: documentType || undefined,
          pattern,
          picker_mode: pickerMode,
        },
      });

      const el = input.parentElement?.querySelector('.catalog-entries');
      if (!el) {
        throw new Error('cannot find list element');
      }

      replaceEl(el as HTMLElement, catalog, '.catalog-entries');
    });
  }

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
  }
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
