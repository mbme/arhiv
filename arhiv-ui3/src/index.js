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
}

window.arhiv_ui = {
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
}

window.utils = {
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
}
