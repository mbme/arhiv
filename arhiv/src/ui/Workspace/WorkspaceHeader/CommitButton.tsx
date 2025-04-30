import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';
import { showToast } from 'components/Toaster';
import { dispatchDocumentChangeEvent } from 'Workspace/documentChangeUtils';

interface CommitButtonProps {
  disabled?: boolean;
}
export function CommitButton({ disabled }: CommitButtonProps) {
  const { error, inProgress, triggerRefresh } = useQuery(
    (abortSignal) => RPC.Commit({}, abortSignal),
    {
      refreshOnMount: false,
      onSuccess({ committedIds }) {
        dispatchDocumentChangeEvent(committedIds);

        showToast({
          level: 'info',
          message: `Committed ${committedIds.length} documents`,
        });
      },
      onError(error) {
        showToast({
          level: 'warn',
          message: `Failed to commit: ${String(error)}`,
        });
      },
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
