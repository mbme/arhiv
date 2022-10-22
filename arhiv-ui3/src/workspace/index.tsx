// Must be the first import
if (process.env.NODE_ENV === 'development') {
  require('preact/debug');
}

import { enableMapSet } from 'immer';
import { render } from 'preact';

import '../../scripts/v-editor';

import { ComponentsDemo } from './components/ComponentsDemo';
import { Workspace } from './Workspace';

enableMapSet();

const renderRoot = document.querySelector('main');
if (!renderRoot) {
  throw new Error('render root not found');
}

if (process.env.NODE_ENV === 'development' && location.search.includes('DEMO')) {
  render(<ComponentsDemo />, renderRoot);
} else {
  render(<Workspace />, renderRoot);
}
