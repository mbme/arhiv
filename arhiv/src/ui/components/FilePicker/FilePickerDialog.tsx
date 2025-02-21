import { useState } from 'react';
import { DocumentId, FileEntry } from 'dto';
import { FilePicker } from 'components/FilePicker/FilePicker';
import { FilePickerConfirmationDialog } from 'components/FilePicker/FilePickerConfirmationDialog';
import { Dialog } from 'components/Dialog';

type Props = {
  onAssetCreated: (id: DocumentId) => void;
  onCancel: () => void;
};
export function FilePickerDialog({ onAssetCreated, onCancel }: Props) {
  const [selectedFile, setSelectedFile] = useState<FileEntry>();

  if (selectedFile) {
    return (
      <FilePickerConfirmationDialog
        filePath={selectedFile.path}
        size={selectedFile.size}
        onAssetCreated={onAssetCreated}
        onCancel={() => {
          setSelectedFile(undefined);
        }}
      />
    );
  }

  return (
    <Dialog onHide={onCancel} title="Create asset">
      <FilePicker onFileSelected={setSelectedFile} />
    </Dialog>
  );
}
