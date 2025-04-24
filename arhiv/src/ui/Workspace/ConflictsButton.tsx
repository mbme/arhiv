import { Callback } from 'utils';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';
import { useDocumentChangeHandler } from 'Workspace/documentChangeUtils';

type Props = {
  onClick: Callback;
};
export function ConflictsButton({ onClick }: Props) {
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
      onClick={onClick}
    >
      <span className="hidden xs:inline">{result?.conflictsCount} conflicts</span>
    </Button>
  );
}
