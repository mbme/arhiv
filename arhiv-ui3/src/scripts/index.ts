import {
  autoGrowTextarea,
  call_action,
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

    if (!id) {
      return;
    }

    console.log('Selected attachment', id);

    return copyTextToClipboard(id, 'attachment id');
  }

  async render_catalog(filter: Obj, parentCollection = '', pickerMode: boolean): Promise<string> {
    const catalog = await call_action({
      renderCatalog: {
        parent_collection: parentCollection || undefined,
        filter,
        picker_mode: pickerMode,
      },
    });

    return catalog as string;
  }

  async search_catalog(documentType = '', pattern: string, parentCollection = '', pickerMode: boolean): Promise<string> {
    const catalog = await call_action({
      searchCatalog: {
        parent_collection: parentCollection || undefined,
        document_type: documentType || undefined,
        pattern,
        picker_mode: pickerMode,
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

  async render_delete_document_confirmation_dialog(id: string, parentCollection = '') {
    const dialog = await call_action({
      renderDeleteDocumentConfirmationDialog: {
        id,
        parent_collection: parentCollection || undefined,
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

  initCatalogLoadMore = (button: HTMLButtonElement, filter: Obj, parentCollection = '', pickerMode: boolean) => {
    button.addEventListener('click', async () => {
      const catalog = await this.render_catalog(filter, parentCollection, pickerMode);

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

      const catalog = await this.search_catalog(documentType, pattern, parentCollection, pickerMode);

      const listEl = input.parentElement?.querySelector('ul');
      if (!listEl) {
        throw new Error('cannot find list element');
      }

      replaceEl(listEl, catalog, 'ul');
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
