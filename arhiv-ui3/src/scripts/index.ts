import { copyTextToClipboard, formDataToObj, isEqualFormData, Obj, replaceEl } from './utils';

type Document = {
  id: string,
  data: Record<string, string | undefined>,
};

async function call_action(action: Record<string, unknown>): Promise<unknown> {
  try {
    const response = await fetch('/rpc', {
      method: 'POST',
      cache: 'no-cache',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(action),
    });

    if (!response.ok) {
      throw new Error(`action failed: ${response.status}`);
    }

    return response.json();
  } catch (e) {
    console.error(e);
    alert(e);

    throw e;
  }
}

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

  async archive_document(id: string, archive: boolean, url: string) {
    await call_action({
      archive: { id, archive }
    });
    window.location.assign(url);
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

  async render_catalog(filter: Obj, parent_collection = '', el: HTMLElement) {
    const catalog = await call_action({
      renderCatalog: {
        parent_collection: parent_collection.trim() || undefined,
        filter,
      },
    });

    replaceEl(el, catalog as string);
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
  }

  autoGrowTextarea = (textarea: HTMLTextAreaElement) => {
    const parent = textarea.parentElement;
    if (!parent) {
      console.error("Textarea doesn't have a parent element");
      return;
    }

    const updateHeight = () => {
      // preserve height between updates
      parent.style.height = `${parent.scrollHeight}px`;

      textarea.style.height = 'auto';
      textarea.style.height = `${textarea.scrollHeight}px`;

      parent.style.height = 'auto';
    };

    updateHeight();

    textarea.addEventListener('input', updateHeight, { passive: true });
    window.addEventListener('resize', updateHeight, { passive: true });
  }

}

declare global {
  interface Window {
    arhiv_ui: ArhivUI
  }
}

window.arhiv_ui = new ArhivUI();
