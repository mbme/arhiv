import { DocumentId } from '../dto';
import { Callback, formatBytes } from '../utils/index';
import { useQuery } from '../utils/hooks';
import { uploadFile } from '../utils/network';
import { Dialog } from './Dialog';
import { Button } from './Button';
import { QueryError } from './QueryError';
import { ImageFilePreview } from './ImageFilePreview';

type Props = {
  file: File;
  onSuccess: (id: DocumentId) => void;
  onCancel: Callback;
};
export function ImageUploadDialog({ file, onSuccess, onCancel }: Props) {
  const { error, inProgress, triggerRefresh } = useQuery(
    async (abortSignal) => {
      const result = await uploadFile([file], abortSignal);
      if (result.error) {
        throw new Error(result.error);
      }

      return result.ids[0]!;
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
