import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { Button } from 'components/Button';

export function CommitOrSyncButton() {
  const { error, inProgress, triggerRefresh } = useQuery(
    (abortSignal) => RPC.CommitOrSync({}, abortSignal),
    {
      refreshOnMount: false,
    },
  );

  const hasStagedChanges = false;

  return (
    <Button
      variant="text"
      leadingIcon={hasStagedChanges ? 'save-all' : 'sync'}
      busy={inProgress}
      onClick={triggerRefresh}
      trailingIcon={error ? 'error-triangle' : undefined}
      title={error ? `${hasStagedChanges ? 'Save' : 'Sync'} failed` : undefined}
    >
      {hasStagedChanges ? 'Save' : 'Sync'}
    </Button>
  );
}
