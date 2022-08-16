import { useEffect, useState } from 'preact/hooks';
import { Catalog } from './Catalog/Catalog';
import { DocumentEditor } from './DocumentEditor/DocumentEditor';
import { DocumentViewer } from './DocumentViewer/DocumentViewer';

export function Workspace() {
  const [documentId, setDocumentId] = useState<string>();
  const [edit, setEdit] = useState(false);

  useEffect(() => {
    if (!documentId) {
      setEdit(false);
    }
  }, [documentId]);

  return (
    <div className="max-w-lg bg-white px-4 py-2 max-h-screen overflow-auto">
      <Catalog hidden={Boolean(documentId)} onDocumentSelected={setDocumentId} />

      {documentId && !edit && (
        <DocumentViewer
          key={documentId}
          documentId={documentId}
          onClose={() => setDocumentId(undefined)}
          onEdit={() => setEdit(true)}
        />
      )}

      {documentId && edit && (
        <DocumentEditor
          key={documentId}
          documentId={documentId}
          onSave={() => setEdit(false)}
          onCancel={() => setEdit(false)}
        />
      )}
    </div>
  );
}
