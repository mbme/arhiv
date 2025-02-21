import { DocumentId } from 'dto';
import { Callback, formatBytes } from 'utils';
import { useQuery } from 'utils/hooks';
import { uploadFile } from 'utils/network';
import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';
import { QueryError } from 'components/QueryError';
import { ImageFilePreview } from 'components/ImageFilePreview';

type Props = {
  file: File;
  onSuccess: (id: DocumentId) => void;
  onCancel: Callback;
};
export function ImageUploadDialog({ file, onSuccess, onCancel }: Props) {
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
    if (!inProgress || window.confirm('Image upload is in progress. Are you sure?')) {
      onCancel();
    }
  };

  const buttons = (
    <Button type="button" variant="primary" busy={inProgress} onClick={triggerRefresh}>
      Create asset
    </Button>
  );

  return (
    <Dialog onHide={onHide} title="Upload an image" buttons={buttons}>
      <div className="flex justify-between mb-2">
        <div className="font-mono">
          {file.name} [{file.type}] <span className="text-gray-500">{formatBytes(file.size)}</span>
        </div>
      </div>

      <ImageFilePreview file={file} />

      {error && <QueryError error={error} />}
    </Dialog>
  );
}
