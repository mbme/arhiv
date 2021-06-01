window.arhiv_ui = {
  go_back(fallback = '/') {
    if (history.length > 2) {
      history.back();
    } else {
      window.location = fallback;
    }
  }
}
