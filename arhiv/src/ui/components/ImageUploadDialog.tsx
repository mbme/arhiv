import { useMemo } from 'react';
import { DocumentId } from 'dto';
import { Callback, formatBytes } from 'utils';
import { useQuery } from 'utils/hooks';
import { uploadFile } from 'utils/rpc';
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
      return uploadFile(file, abortSignal);
    },
    {
      refreshOnMount: false,
      onSuccess(result) {
        onSuccess(result);
      },
    },
  );

  const onHide = () => {
    if (!inProgress || window.confirm('File upload is in progress. Are you sure?')) {
      onCancel();
    }
  };

  const buttons = (
    <Button type="button" variant="primary" busy={inProgress} onClick={triggerRefresh}>
      Create attachment
    </Button>
  );

  return (
    <Dialog onHide={onHide} title="Upload a file" buttons={buttons}>
      <div className="flex justify-between mb-2">
        <div className="font-mono">
          {file.name} [{file.type}] <span className="text-gray-500">{formatBytes(file.size)}</span>
        </div>

        <Link url={imgUrl}>Open in a new tab</Link>
      </div>

      <img src={imgUrl} />

      {error && <QueryError error={error} />}
    </Dialog>
  );
}
