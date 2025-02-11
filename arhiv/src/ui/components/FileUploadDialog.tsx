import { useState } from 'react';
import { DocumentId } from 'dto';
import { formatBytes } from 'utils';
import { useQuery } from 'utils/hooks';
import { uploadFile } from 'utils/network';
import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';
import { QueryError } from 'components/QueryError';
import { ImageFilePreview } from './ImageFilePreview';

type Props = {
  onAttachmentCreated: (id: DocumentId) => void;
  onCancel: () => void;
};
export function FileUploadDialog({ onAttachmentCreated, onCancel }: Props) {
  const [file, setFile] = useState<File>();

  const { error, inProgress, triggerRefresh } = useQuery(
    async (abortSignal) => {
      if (!file) {
        throw new Error('File not selected');
      }

      return uploadFile(file, abortSignal);
    },
    {
      refreshOnMount: false,
      onSuccess(result) {
        onAttachmentCreated(result);
      },
    },
  );

  const onHide = () => {
    if (!inProgress || window.confirm('File upload is in progress. Are you sure?')) {
      onCancel();
    }
  };

  const buttons = (
    <>
      <Button variant="simple" onClick={onCancel} disabled={inProgress}>
        Cancel
      </Button>

      <Button variant="primary" busy={inProgress} disabled={!file} onClick={triggerRefresh}>
        Create attachment
      </Button>
    </>
  );

  return (
    <Dialog onHide={onHide} title="Upload a file" buttons={buttons}>
      <form>
        <label className="inline-block bg-blue-500 text-white py-2 px-4 rounded-sm cursor-pointer hover:bg-blue-600">
          Choose a file
          <input
            className="hidden"
            type="file"
            required
            onChange={(e) => {
              setFile(e.currentTarget.files?.[0] ?? undefined);
            }}
          />
        </label>

        {file && (
          <div className="font-mono">
            {file.name} [{file.type}]{' '}
            <span className="text-gray-500">{formatBytes(file.size)}</span>
          </div>
        )}

        {file && <ImageFilePreview file={file} className="max-w-full mx-auto" />}

        {error && <QueryError error={error} />}
      </form>
    </Dialog>
  );
}
