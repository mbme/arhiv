import { useState } from 'react';
import { DocumentEditor } from './DocumentEditor/DocumentEditor';
import { DocumentViewer } from './DocumentViewer/DocumentViewer';

type DocumentCardProps = {
  documentId: string;
};
export function DocumentCard({ documentId }: DocumentCardProps) {
  const [edit, setEdit] = useState(false);

  return (
    <>
      {edit ? (
        <DocumentEditor
          key={documentId}
          documentId={documentId}
          onSave={() => setEdit(false)}
          onCancel={() => setEdit(false)}
        />
      ) : (
        <DocumentViewer key={documentId} documentId={documentId} onEdit={() => setEdit(true)} />
      )}
    </>
  );
}
