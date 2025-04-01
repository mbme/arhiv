import { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { effect } from '@preact/signals-core';
import { storage } from 'utils/storage';
import { appController } from 'controller';
import { ComponentsDemo } from 'ComponentsDemo';
import { Workspace } from 'Workspace/Workspace';
import { CreateArhiv } from 'Login/CreateArhiv';
import { UnlockArhiv } from 'Login/UnlockArhiv';
import { ImportArhivKey } from 'Login/ImportArhivKey';
import { Button } from 'components/Button';
import { QueryError } from 'components/QueryError';
import { ErrorBoundary } from 'components/ErrorBoundary';
import { showToast, Toaster } from 'components/Toaster';

window.APP = appController;

effect(() => {
  document.documentElement.classList.toggle('dark', appController.$theme.value === 'dark');
});

const renderError = (error: unknown) => (
  <>
    <div className="px-8 py-4">
      <QueryError error={error} />
    </div>

    <div className="flex flex-row justify-center mt-8">
      <Button
        onClick={() => {
          storage.clear();
          window.location.reload();
        }}
        variant="primary"
        alarming
      >
        Clear storage and refresh the page
      </Button>
    </div>
  </>
);

function renderView() {
  if (window.CONFIG.arhivMissing) {
    return <CreateArhiv />;
  } else if (window.CONFIG.arhivKeyMissing) {
    return <ImportArhivKey />;
  } else if (window.CONFIG.arhivLocked) {
    return <UnlockArhiv />;
  } else if (process.env.NODE_ENV === 'development' && location.search.includes('DEMO')) {
    return <ComponentsDemo />;
  } else {
    return <Workspace />;
  }
}

function App() {
  const [view, setView] = useState(renderView);

  useEffect(() => {
    const onPopState = () => {
      setView(renderView());
    };
    window.addEventListener('popstate', onPopState);

    return () => {
      window.removeEventListener('popstate', onPopState);
    };
  }, []);

  useEffect(() => {
    const TOAST_TIMEOUT = 30 * 60 * 1000; // 30 min

    const onError = (e: ErrorEvent) => {
      showToast({
        level: 'warn',
        message: `Caught error: ${e.message} at ${e.filename}:${e.lineno}`,
        timeoutMs: TOAST_TIMEOUT,
      });
    };
    window.addEventListener('error', onError);

    const onUnhandledPromiseRejection = (e: PromiseRejectionEvent) => {
      showToast({
        level: 'warn',
        message: `Unhandled Promise Rejection: ${e.reason}`,
        timeoutMs: TOAST_TIMEOUT,
      });
    };

    window.addEventListener('unhandledrejection', onUnhandledPromiseRejection);

    return () => {
      window.removeEventListener('error', onError);
      window.removeEventListener('unhandledrejection', onUnhandledPromiseRejection);
    };
  }, []);

  return (
    <ErrorBoundary renderError={renderError}>
      {view}

      <Toaster />
    </ErrorBoundary>
  );
}

const rootEl = document.querySelector('main');
if (!rootEl) {
  throw new Error('render root not found');
}

const root = createRoot(rootEl);
root.render(<App />);
