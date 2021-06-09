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

  delete_document(id) {
    return call_action({
      Delete: { id }
    });
  },

  archive_document(id, archive) {
    return call_action({
      Archive: { id, archive }
    });
  },
}
