import { useState } from 'react';
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
  const [moveFile, setMoveFile] = useState(false);

  const { error, inProgress, triggerRefresh } = useQuery(
    async (abortSignal) => {
      const { id } = await RPC.CreateAttachment({ filePath, moveFile }, abortSignal);
      onAttachmentCreated(id);
    },
    {
      refreshOnMount: false,
    },
  );

  const onHide = () => {
    if (!inProgress) {
      onCancel();
    }
  };

  const buttons = (
    <>
      <Button variant="simple" onClick={onCancel} disabled={inProgress}>
        Cancel
      </Button>

      <Button variant="primary" busy={inProgress} onClick={triggerRefresh}>
        Create attachment
      </Button>
    </>
  );

  return (
    <Dialog onHide={onHide} title="Add file" buttons={buttons}>
      <div className="mb-6">
        Do you really want to create attachment from the file <code>{filePath}</code> of size{' '}
        <b>{formatBytes(size)}</b>?
      </div>

      <form
        className="form"
        onSubmit={(e) => {
          e.preventDefault();
        }}
      >
        <label className="flex items-center gap-2 text-sm cursor-pointer">
          <input
            name="move_file"
            type="checkbox"
            className="field"
            checked={moveFile}
            onChange={() => {
              setMoveFile(!moveFile);
            }}
          />
          Remove original file
        </label>
      </form>

      {error && <QueryError error={error} />}
    </Dialog>
  );
}
