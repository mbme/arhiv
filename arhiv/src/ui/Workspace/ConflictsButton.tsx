import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';
import { useDocumentChangeHandler } from 'Workspace/documentChangeUtils';

export function ConflictsButton() {
  const { result, error, inProgress, triggerRefresh } = useQuery((abortSignal) =>
    RPC.CountConflicts({}, abortSignal),
  );

  useDocumentChangeHandler(() => {
    triggerRefresh();
  });

  if (result && result.conflictsCount === 0) {
    return null;
  }

  return (
    <Button
      variant="text"
      alarming
      leadingIcon="error-triangle"
      busy={inProgress}
      title={error ? 'Failed to fetch conflicts' : undefined}
    >
      <span className="hidden xs:inline">{result?.conflictsCount} conflicts</span>
    </Button>
  );
}
