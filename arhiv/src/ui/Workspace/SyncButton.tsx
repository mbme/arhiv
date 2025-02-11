import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';

interface SyncButtonProps {
  disabled?: boolean;
}
export function SyncButton({ disabled }: SyncButtonProps) {
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
