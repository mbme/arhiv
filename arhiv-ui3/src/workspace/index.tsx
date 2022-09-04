import ReactDOM from 'react-dom/client';
import { ComponentsDemo } from './components/ComponentsDemo';
import { Workspace } from './Workspace';

const renderRoot = document.querySelector('main');
if (!renderRoot) {
  throw new Error('render root not found');
}

const root = ReactDOM.createRoot(renderRoot);
if (process.env.NODE_ENV === 'development' && location.search.includes('DEMO')) {
  root.render(<ComponentsDemo />);
} else {
  root.render(<Workspace />);
}
