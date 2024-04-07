import { createRoot } from 'react-dom/client';
import { App } from 'App';
import { ComponentsDemo } from 'ComponentsDemo';
import { CreateArhiv } from 'CreateArhiv';

const rootEl = document.querySelector('main');
if (!rootEl) {
  throw new Error('render root not found');
}
const root = createRoot(rootEl);

function renderApp() {
  if (window.CREATE_ARHIV) {
    root.render(<CreateArhiv />);
  } else if (process.env.NODE_ENV === 'development' && location.search.includes('DEMO')) {
    root.render(<ComponentsDemo />);
  } else {
    root.render(<App />);
  }
}

renderApp();

window.addEventListener('popstate', renderApp);
