// Must be the first import
if (process.env.NODE_ENV === 'development') {
  require('preact/debug');
}

import { enableMapSet } from 'immer';
import { render } from 'preact';

// register web components
import './components/Form/v-editor';
import './components/Form/v-ref-input';

import { ComponentsDemo } from './components/ComponentsDemo';
import { Workspace } from './Workspace/Workspace';

enableMapSet();

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
