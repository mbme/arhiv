// Must be the first import
if (process.env.NODE_ENV === 'development') {
  require('preact/debug');
}

import { render } from 'preact';

// register web components
import 'components/Form/v-form-field';

import { ComponentsDemo } from 'components/ComponentsDemo';
import { Workspace } from 'Workspace/Workspace';

function renderApp() {
  const renderRoot = document.querySelector('main');
  if (!renderRoot) {
    throw new Error('render root not found');
  }

  if (process.env.NODE_ENV === 'development' && location.search.includes('DEMO')) {
    render(<ComponentsDemo />, renderRoot);
  } else {
    render(<Workspace />, renderRoot);
  }
}

renderApp();

window.addEventListener('popstate', renderApp);
