import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';

interface CommitButtonProps {
  disabled?: boolean;
}
export function CommitButton({ disabled }: CommitButtonProps) {
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
      disabled={disabled}
      onClick={triggerRefresh}
      trailingIcon={error ? 'error-triangle' : undefined}
      title={error ? 'Commit failed' : undefined}
    >
      <span className="hidden md:inline">Save</span>
    </Button>
  );
}
