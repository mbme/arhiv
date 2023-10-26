import { useEffect } from 'react';
import { usePageVisibilityTracker, useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { BazaEvent } from 'dto';
import { Button } from 'components/Button';

const useIsArhivModified = (): boolean => {
  const { result, triggerRefresh, requestTs } = useQuery((abortSignal) =>
    RPC.GetIsModified({}, abortSignal),
  );

  useEffect(() => {
    const bazaEvents = new EventSource(`${window.BASE_PATH}/events`);

    const rpcEventHandler = (e: MessageEvent<string>) => {
      const event = JSON.parse(e.data) as BazaEvent;

      switch (event.typeName) {
        case 'DocumentStaged':
        case 'DocumentsCommitted': {
          triggerRefresh();
        }
      }
    };

    bazaEvents.addEventListener('message', rpcEventHandler);

    return () => {
      bazaEvents.removeEventListener('message', rpcEventHandler);
      bazaEvents.close();
    };
  }, [triggerRefresh]);

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

  return result?.isModified ?? false;
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
      Save
    </Button>
  );
}

function SyncButton() {
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
    >
      Sync
    </Button>
  );
}

export function CommitOrSyncButton() {
  const isArhivModified = useIsArhivModified();

  if (isArhivModified) {
    return <CommitButton />;
  }

  return <SyncButton />;
}
