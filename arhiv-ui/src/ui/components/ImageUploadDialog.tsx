import { useMemo } from 'react';
import { DocumentId } from 'dto';
import { Callback, fileAsBase64, formatBytes } from 'utils';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';
import { Link } from 'components/Link';
import { QueryError } from 'components/QueryError';

type Props = {
  file: File;
  onSuccess: (id: DocumentId) => void;
  onCancel: Callback;
};
export function ImageUploadDialog({ file, onSuccess, onCancel }: Props) {
  const imgUrl = useMemo(() => URL.createObjectURL(file), [file]);

  const { error, inProgress, triggerRefresh } = useQuery(
    async (abortSignal) => {
      const base64Data = await fileAsBase64(file);

      return RPC.UploadFile({ fileName: file.name, base64Data }, abortSignal);
    },
    {
      refreshOnMount: false,
      onSuccess(result) {
        onSuccess(result.id);
      },
    }
  );

  const onHide = () => {
    if (!inProgress || window.confirm('File upload is in progress. Are you sure?')) {
      onCancel();
    }
  };

  return (
    <Dialog onHide={onHide} title="Upload a file">
      <div className="modal-content">
        <div className="font-mono">
          {file.name} [{file.type}] <span className="text-gray-500">{formatBytes(file.size)}</span>
        </div>

        <Link url={imgUrl}>
          <img src={imgUrl} />
        </Link>

        {error && <QueryError error={error} />}
      </div>

      <div className="modal-buttons">
        <Button type="button" variant="primary" busy={inProgress} onClick={triggerRefresh}>
          Create attachment
        </Button>
      </div>
    </Dialog>
  );
}
