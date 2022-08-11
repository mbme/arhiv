import { render } from 'preact';
import { Workspace } from './components/Workspace';

const renderRoot = document.querySelector('main');
if (!renderRoot) {
  throw new Error('render root not found');
}

render(<Workspace />, renderRoot);
