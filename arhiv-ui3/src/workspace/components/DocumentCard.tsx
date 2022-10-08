import { useState } from 'preact/hooks';
import { DocumentEditor } from './DocumentEditor/DocumentEditor';
import { DocumentViewer } from './DocumentViewer/DocumentViewer';

type Props = {
  documentId: string;
  query?: string;
  page?: number;
};
export function DocumentCard({ documentId, query, page }: Props) {
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

  return (
    <DocumentViewer
      key={documentId}
      documentId={documentId}
      onEdit={() => setEdit(true)}
      query={query}
      page={page}
    />
  );
}
