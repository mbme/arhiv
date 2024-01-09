import { Workspace } from 'Workspace/Workspace';
import { Button } from 'components/Button';
import { ErrorBoundary } from 'components/ErrorBoundary';
import { QueryError } from 'components/QueryError';
import { storage } from 'utils/storage';

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

export function App() {
  return (
    <ErrorBoundary renderError={renderError}>
      <Workspace />
    </ErrorBoundary>
  );
}
