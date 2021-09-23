import {
  autoGrowTextarea,
  call_action,
  copyTextToClipboard,
  formDataToObj,
  isEqualFormData,
  Obj,
  renderModal,
  replaceEl,
  updateQueryParam,
} from './utils';
import { initDataJS } from './data-js';

type Document = {
  id: string,
  data: Record<string, string | undefined>,
};

class ArhivUI {
  go_back(fallback = '/') {
    if (history.length > 2) {
      history.back();
    } else {
      window.location.assign(fallback);
    }
  }

  async delete_document(id: string, urlOnDelete: string) {
    await call_action({
      delete: { id }
    });

    window.location.assign(urlOnDelete);
  }

  async archive_document(id: string, archive: boolean) {
    await call_action({
      archive: { id, archive }
    });

    window.location.reload();
  }

  async save_document(document: Document) {
    await call_action({
      save: { document }
    });
  }

  async pick_attachment() {
    const id = await call_action({
      pickAttachment: { }
    }) as string;

    if (id) {
      console.log('Selected attachment', id);

      copyTextToClipboard(id);
    }
  }

  async render_catalog(filter: Obj, parent_collection = ''): Promise<string> {
    const catalog = await call_action({
      renderCatalog: {
        parent_collection: parent_collection || undefined,
        filter,
      },
    });

    return catalog as string;
  }

  async search_catalog(document_type = '', pattern: string, parent_collection = ''): Promise<string> {
    const catalog = await call_action({
      searchCatalog: {
        parent_collection: parent_collection || undefined,
        document_type: document_type || undefined,
        pattern,
      },
    });

    return catalog as string;
  }

  async render_archive_document_confirmation_dialog(id: string) {
    const dialog = await call_action({
      renderArchiveDocumentConfirmationDialog: { id },
    });

    renderModal(dialog as string);
  }

  async render_delete_document_confirmation_dialog(id: string, parent_collection = '') {
    const dialog = await call_action({
      renderDeleteDocumentConfirmationDialog: {
        id,
        parent_collection: parent_collection || undefined,
      },
    });

    renderModal(dialog as string);
  }

  async render_pick_document_modal() {
    const dialog = await call_action({
      renderPickDocumentModal: {},
    });

    renderModal(dialog as string);
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

    form.addEventListener('submit', (event: Event) => {
      event.preventDefault();

      void this.save_document({
        ...originalDocument,
        data: formDataToObj(new FormData(form)),
      }).then(() => {
        window.removeEventListener('beforeunload', onBeforeUnload, { capture: true });
        window.location.assign(urlOnSave);
      });
    });

    form.querySelectorAll('textarea').forEach(autoGrowTextarea);
  }

  initCatalogLoadMore = (button: HTMLButtonElement, filter: Obj, parent_collection = '') => {
    button.addEventListener('click', async () => {
      const catalog = await this.render_catalog(filter, parent_collection);

      if (!button.parentElement) {
        throw new Error("button doesn't have a parent");
      }

      replaceEl(button.parentElement, catalog, 'ul > li');
    });
  }

  initCatalogSearch = (input: HTMLInputElement, document_type = '', parent_collection = '') => {
    input.addEventListener('change', async () => {
      const pattern = input.value;

      updateQueryParam('pattern', pattern);

      const catalog = await this.search_catalog(document_type, pattern, parent_collection);

      const listEl = input.parentElement?.querySelector('ul');
      if (!listEl) {
        throw new Error('cannot find list element');
      }

      replaceEl(listEl, catalog, 'ul');
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
