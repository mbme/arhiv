import { useState } from 'react';
import { DocumentId } from 'dto';
import { Callback, formatBytes } from 'utils';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';
import { Dialog } from 'components/Dialog';
import { QueryError } from 'components/QueryError';

type Props = {
  filePath: string;
  size: number;
  onAssetCreated: (id: DocumentId) => void;
  onCancel: Callback;
};
export function FilePickerConfirmationDialog({ filePath, size, onAssetCreated, onCancel }: Props) {
  const [removeFile, setRemoveFile] = useState(false);

  const { error, inProgress, triggerRefresh } = useQuery(
    async (abortSignal) => {
      const { id } = await RPC.CreateAsset({ filePath, removeFile }, abortSignal);
      onAssetCreated(id);
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
        Create asset
      </Button>
    </>
  );

  return (
    <Dialog onHide={onHide} title="Create asset" buttons={buttons}>
      <div className="mb-6">
        Do you really want to create asset from the file <code>{filePath}</code> of size{' '}
        <b>{formatBytes(size)}</b>?
      </div>

      <form
        onSubmit={(e) => {
          e.preventDefault();
        }}
      >
        <label className="flex items-center gap-2 text-sm cursor-pointer">
          <input
            name="remove_file"
            type="checkbox"
            checked={removeFile}
            onChange={() => {
              setRemoveFile(!removeFile);
            }}
          />
          Remove original file
        </label>
      </form>

      {error && <QueryError error={error} />}
    </Dialog>
  );
}
