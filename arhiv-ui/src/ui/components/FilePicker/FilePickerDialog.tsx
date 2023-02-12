import { useState } from 'preact/hooks';
import { DocumentId, FileEntry } from 'dto';
import { FilePicker } from 'components/FilePicker/FilePicker';
import { FilePickerConfirmationDialog } from 'components/FilePicker/FilePickerConfirmationDialog';
import { Dialog } from 'components/Dialog';

type Props = {
  onAttachmentCreated: (id: DocumentId) => void;
  onCancel: () => void;
};
export function FilePickerDialog({ onAttachmentCreated, onCancel }: Props) {
  const [selectedFile, setSelectedFile] = useState<FileEntry>();

  if (selectedFile) {
    return (
      <FilePickerConfirmationDialog
        filePath={selectedFile.path}
        size={selectedFile.size}
        onAttachmentCreated={onAttachmentCreated}
        onCancel={() => setSelectedFile(undefined)}
      />
    );
  }

  return (
    <Dialog onHide={onCancel} title="Add file">
      <div className="modal-content">
        <FilePicker onFileSelected={setSelectedFile} />
      </div>
    </Dialog>
  );
}
