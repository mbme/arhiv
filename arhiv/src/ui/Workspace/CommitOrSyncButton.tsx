import { useEffect } from 'react';
import { MUTABLE_API_REQUESTS } from 'dto';
import { usePageVisibilityTracker, useQuery } from 'utils/hooks';
import { RPC, RPCEvent } from 'utils/rpc';
import { Button } from 'components/Button';

const useIsArhivModified = (): boolean => {
  const { result, triggerRefresh, requestTs } = useQuery((abortSignal) =>
    RPC.GetIsModified({}, abortSignal),
  );

  useEffect(() => {
    const rpcEventHandler = (e: Event) => {
      const typeName = (e as RPCEvent).eventType;

      if (MUTABLE_API_REQUESTS.includes(typeName)) {
        triggerRefresh();
      }
    };

    document.addEventListener('rpcEvent', rpcEventHandler);

    return () => {
      document.removeEventListener('rpcEvent', rpcEventHandler);
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

export function CommitOrSyncButton() {
  const isArhivModified = useIsArhivModified();

  const { error, inProgress, triggerRefresh } = useQuery(
    (abortSignal) => RPC.CommitOrSync({}, abortSignal),
    {
      refreshOnMount: false,
    },
  );

  return (
    <Button
      variant="text"
      leadingIcon={isArhivModified ? 'save-all' : 'sync'}
      busy={inProgress}
      onClick={triggerRefresh}
      trailingIcon={error ? 'error-triangle' : undefined}
      title={error ? `${isArhivModified ? 'Save' : 'Sync'} failed` : undefined}
    >
      {isArhivModified ? 'Save' : 'Sync'}
    </Button>
  );
}
