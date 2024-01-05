import { usePageVisibilityTracker, useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { useBazaEvent } from 'baza-events';
import { Button } from 'components/Button';

type SaveState = {
  canCommit: boolean;
  canSync: boolean;
};

const useSaveState = (): SaveState => {
  const { result, triggerRefresh, requestTs } = useQuery((abortSignal) =>
    RPC.GetSaveState({}, abortSignal),
  );

  useBazaEvent((event) => {
    switch (event.typeName) {
      case 'DocumentStaged':
      case 'DocumentsCommitted':
      case 'DocumentLocked':
      case 'DocumentUnlocked': {
        triggerRefresh();
      }
    }
  });

  usePageVisibilityTracker((isPageVisible) => {
    const secondsSinceLastSync = (Date.now() - requestTs) / 1000;

    if (isPageVisible && secondsSinceLastSync > 10 * 60) {
      console.debug(
        'page became visible and %s seconds elapsed since last check, refreshing',
        secondsSinceLastSync,
      );
      triggerRefresh();
    }
  });

  return result ?? { canCommit: false, canSync: false };
};

function CommitButton() {
  const { error, inProgress, triggerRefresh } = useQuery(
    (abortSignal) => RPC.Commit({}, abortSignal),
    {
      refreshOnMount: false,
    },
  );

  return (
    <Button
      variant="text"
      leadingIcon="save-all"
      busy={inProgress}
      onClick={triggerRefresh}
      trailingIcon={error ? 'error-triangle' : undefined}
      title={error ? 'Commit failed' : undefined}
    >
      <span className="hidden md:inline">Save</span>
    </Button>
  );
}

interface SyncButtonProps {
  disabled: boolean;
}
function SyncButton({ disabled }: SyncButtonProps) {
  const { error, inProgress, triggerRefresh } = useQuery(
    (abortSignal) => RPC.Sync({}, abortSignal),
    {
      refreshOnMount: false,
    },
  );

  return (
    <Button
      variant="text"
      leadingIcon="sync"
      busy={inProgress}
      onClick={triggerRefresh}
      trailingIcon={error ? 'error-triangle' : undefined}
      title={error ? 'Sync failed' : undefined}
      disabled={disabled}
    >
      <span className="hidden md:inline">Sync</span>
    </Button>
  );
}

export function CommitOrSyncButton() {
  const { canCommit, canSync } = useSaveState();

  if (canCommit) {
    return <CommitButton />;
  }

  return <SyncButton disabled={!canSync} />;
}
