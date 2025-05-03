import { useState } from 'react';
import { DocumentId } from 'dto';
import { Callback, formatBytes, newId, withoutItems } from 'utils';
import { useQuery } from 'utils/hooks';
import { uploadFile } from 'utils/network';
import { Dialog } from 'components/Dialog';
import { Button, IconButton } from 'components/Button';
import { QueryError } from 'components/QueryError';
import { ImageFilePreview } from 'components/ImageFilePreview';
import { FileInput } from 'components/FileInput';

type FileExt = File & { id: string };

type FileUploaderProps = {
  className?: string;
  file: File;
  onRemove?: Callback;
};
function FileInfo({ className, file, onRemove }: FileUploaderProps) {
  return (
    <li className={className}>
      <div className="font-mono text-sm flex gap-1 sm:gap-4 items-center">
        <span className="break-all">{file.name}</span>
        <span>[{file.type}]</span>
        <span className="text-gray-500 whitespace-nowrap">{formatBytes(file.size)}</span>

        {onRemove && <IconButton icon="x" className="cursor-pointer" onClick={onRemove} />}
      </div>

      <ImageFilePreview file={file} className="max-w-full mx-auto" />
    </li>
  );
}

type FileInfoListProps = {
  files: FileExt[];
  onRemove?: (file: FileExt) => void;
};
function FileInfoList({ files, onRemove }: FileInfoListProps) {
  return (
    <ol className="pl-4 pt-8 list-decimal">
      {files.map((file) => (
        <FileInfo
          key={file.id}
          className="mb-2"
          file={file}
          onRemove={
            onRemove
              ? () => {
                  onRemove(file);
                }
              : undefined
          }
        />
      ))}
    </ol>
  );
}

type FileUploadDialogProps = {
  onClose: (ids: DocumentId[]) => void;
};
export function FileUploadDialog({ onClose }: FileUploadDialogProps) {
  const [files, setFiles] = useState<FileExt[]>([]);

  const { result, error, inProgress, triggerRefresh } = useQuery(
    async (abortSignal) => {
      if (files.length === 0) {
        throw new Error('File not selected');
      }

      return uploadFile(files, abortSignal);
    },
    {
      refreshOnMount: false,
      onSuccess(result) {
        if (!result.error) {
          // skip "results" view if there are no errors
          onClose(result.ids);
        }
      },
    },
  );

  const onHide = () => {
    if (!inProgress || window.confirm('File upload is in progress. Are you sure?')) {
      onClose(result?.ids ?? []);
    }
  };

  if (result) {
    return (
      <Dialog
        onHide={onHide}
        title="Upload files"
        buttons={
          <Button variant="primary" onClick={onHide}>
            Ok
          </Button>
        }
      >
        <h3>
          Uploaded {result.ids.length} files out of {files.length}
        </h3>

        {result.error && <QueryError error={result.error} />}

        <FileInfoList files={files.slice(result.ids.length)} />
      </Dialog>
    );
  }

  return (
    <Dialog
      onHide={onHide}
      title="Upload files"
      buttons={
        <>
          <Button variant="simple" onClick={onHide} disabled={inProgress}>
            Cancel
          </Button>

          <Button
            variant="primary"
            busy={inProgress}
            disabled={files.length === 0}
            onClick={triggerRefresh}
          >
            Create assets
          </Button>
        </>
      }
    >
      <form>
        <FileInput
          label="Choose files"
          variant="primary"
          required
          multiple
          disabled={inProgress}
          onSelected={(newFiles) => {
            const newFilesWithIds = newFiles.map((file) => {
              const fileExt = file as FileExt;
              fileExt.id = newId();

              return fileExt;
            });

            setFiles([...files, ...newFilesWithIds]);
          }}
        />

        <FileInfoList
          files={files}
          onRemove={
            inProgress
              ? undefined
              : (file) => {
                  setFiles(withoutItems(files, file));
                }
          }
        />

        {error && <QueryError error={error} />}
      </form>
    </Dialog>
  );
}
