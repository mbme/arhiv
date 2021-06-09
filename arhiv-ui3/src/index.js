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
        Delete: { id }
      });
      window.location = '/';
    } catch (e) {
      console.error(e);
      alert(e);
    }
  },

  async archive_document(id, archive) {
    try {
      await call_action({
        Archive: { id, archive }
      });
      window.location = `/documents/${id}`;
    } catch (e) {
      console.error(e);
      alert(e);
    }
  },
}
