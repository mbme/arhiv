import { useState } from 'preact/hooks';
import { DocumentId } from 'dto';
import { DocumentEditor } from './DocumentEditor/DocumentEditor';
import { DocumentViewer } from './DocumentViewer/DocumentViewer';

type Props = {
  documentId: DocumentId;
};
export function DocumentCard({ documentId }: Props) {
  const [edit, setEdit] = useState(false);

  if (edit) {
    return (
      <DocumentEditor
        key={documentId}
        documentId={documentId}
        onSave={() => setEdit(false)}
        onCancel={() => setEdit(false)}
      />
    );
  }

  return <DocumentViewer key={documentId} documentId={documentId} onEdit={() => setEdit(true)} />;
}
