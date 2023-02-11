import { DocumentId } from 'dto';
import { Callback, formatBytes } from 'utils';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { Button } from 'components/Button';
import { Dialog } from 'components/Dialog';
import { QueryError } from 'components/QueryError';

type Props = {
  filePath: string;
  size: number;
  onAttachmentCreated: (id: DocumentId) => void;
  onCancel: Callback;
};
export function FilePickerConfirmationDialog({
  filePath,
  size,
  onAttachmentCreated,
  onCancel,
}: Props) {
  const { error, inProgress, triggerRefresh } = useQuery(
    async (abortSignal) => {
      const { id } = await RPC.CreateAttachment({ filePath }, abortSignal);
      onAttachmentCreated(id);
    },
    {
      refreshOnMount: false,
    }
  );

  return (
    <Dialog onHide={onCancel} title="Add file">
      <div className="modal-content">
        <div>
          Do you really want to create attachment from the file <code>{filePath}</code> of size{' '}
          <b>{formatBytes(size)}</b>?
        </div>

        {error && <QueryError error={error} />}
      </div>

      <div className="modal-buttons">
        <Button variant="simple" onClick={onCancel} disabled={inProgress}>
          Cancel
        </Button>

        <Button variant="primary" busy={inProgress} onClick={triggerRefresh}>
          Create attachment
        </Button>
      </div>
    </Dialog>
  );
}
