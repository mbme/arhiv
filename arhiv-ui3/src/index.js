async function call_action(action) {
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
}

const arhiv_ui = {
  go_back(fallback = '/') {
    if (history.length > 2) {
      history.back();
    } else {
      window.location = fallback;
    }
  },

  async delete_document(id) {
    try {
      await call_action({
        delete: { id }
      });
      window.location = '/';
    } catch (e) {
      console.error(e);
      alert(e);

      throw e;
    }
  },

  async archive_document(id, archive) {
    try {
      await call_action({
        archive: { id, archive }
      });
      window.location = `/documents/${id}`;
    } catch (e) {
      console.error(e);
      alert(e);

      throw e;
    }
  },

  async save_document(document) {
    try {
      await call_action({
        save: { document }
      });
    } catch (e) {
      console.error(e);
      alert(e);

      throw e;
    }
  },

  async pick_attachment() {
    try {
      const id = await call_action({
        pickAttachment: { }
      });

      if (id) {
        console.log('Selected attachment', id);

        utils.copyTextToClipboard(id);
      }
    } catch (e) {
      console.error(e);
      alert(e);

      throw e;
    }
  }
};

const utils = {
  formDataToObj(fd) {
    const result = {};

    for (const [key, value] of fd.entries()) {
      result[key] = value;
    }

    return result;
  },

  isEqualFormData(fd1, fd2) {
    const fd1Obj = this.formDataToObj(fd1);
    const fd2Obj = this.formDataToObj(fd2);

    return JSON.stringify(fd1Obj) === JSON.stringify(fd2Obj);
  },

  copyTextToClipboard(text) {
    const writeText = () => {
      navigator.clipboard.writeText(text).catch((e) => {
        console.error('failed to copy text to clipboard', e);
      });
    };

    if (document.hasFocus()) {
      writeText();
    } else {
      window.addEventListener('focus', writeText, { once: true });
    }
  }
};

arhiv_ui.initEditorForm = (form, originalDocument) => {
  const initialFormData = new FormData(form);

  function onBeforeUnload(event) {
    const fd = new FormData(form);

    if (!utils.isEqualFormData(initialFormData, fd)) {
      event.preventDefault();
      return event.returnValue = 'Page has unsaved changes. Are you sure you want to exit?';
    }
  }

  window.addEventListener('beforeunload', onBeforeUnload, { capture: true });

  form.addEventListener('submit', (event) => {
    event.preventDefault();

    arhiv_ui.save_document({
      ...originalDocument,
      data: utils.formDataToObj(new FormData(form)),
    }).then(() => {
      window.removeEventListener('beforeunload', onBeforeUnload, { capture: true });
      window.location = `/documents/${originalDocument.id}`;
    });
  });
};

arhiv_ui.autoGrowTextarea = (textarea) => {
  const parent = textarea.parentElement;

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
};

window.utils = utils;
window.arhiv_ui = arhiv_ui;
