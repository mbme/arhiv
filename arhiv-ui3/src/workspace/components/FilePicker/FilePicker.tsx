import { useState } from 'react';
import { DirEntry } from '../../dto';
import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { Icon } from '../Icon';
import { QueryError } from '../QueryError';
import { FilePickerConfirmationDialog } from './FilePickerConfirmationDialog';
import { FilePickerEntry } from './FilePickerEntry';
import { FilePickerHead } from './FilePickerHead';

type Props = {
  onAttachmentCreated: (id: string) => void;
};
export function FilePicker({ onAttachmentCreated }: Props) {
  const [showHidden, setShowHidden] = useState(false);
  const [dir, setDir] = useState<string>();
  const [selectedFile, setSelectedFile] = useState<DirEntry>();

  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.ListDir({ dir, showHidden }, abortSignal),
    {
      refreshIfChange: [dir, showHidden],
    }
  );

  return (
    <div>
      <div className="flex justify-between mb-6 mx-4">
        <FilePickerHead dir={result?.dir ?? dir} entries={result?.entries} />

        <form className="form" onSubmit={(e) => e.preventDefault()}>
          <label className="flex items-center gap-2 text-sm cursor-pointer">
            <input
              name="show_hidden"
              type="checkbox"
              className="field"
              checked={showHidden}
              onChange={() => setShowHidden(!showHidden)}
            />
            Show hidden
          </label>
        </form>
      </div>

      {error && <QueryError error={error} />}

      {inProgress && <Icon variant="spinner" className="mb-8" />}

      {result?.entries.map((entry) => (
        <FilePickerEntry
          key={entry.name}
          entry={entry}
          onClick={() => {
            if (entry.typeName === 'Dir') {
              setDir(entry.path);
            }

            if (entry.typeName === 'File') {
              setSelectedFile(entry);
            }
          }}
        />
      ))}

      {selectedFile?.typeName === 'File' && (
        <FilePickerConfirmationDialog
          filePath={selectedFile.path}
          size={selectedFile.size}
          onAttachmentCreated={onAttachmentCreated}
          onCancel={() => setSelectedFile(undefined)}
        />
      )}
    </div>
  );
}
