import { useEffect, useState } from 'preact/hooks';
import { Catalog } from './Catalog/Catalog';
import { DocumentEditor } from './DocumentEditor/DocumentEditor';
import { DocumentViewer } from './DocumentViewer/DocumentViewer';

export function CatalogCard() {
  const [documentId, setDocumentId] = useState<string>();
  const [edit, setEdit] = useState(false);

  useEffect(() => {
    if (!documentId) {
      setEdit(false);
    }
  }, [documentId]);

  return (
    <>
      {!documentId && <Catalog onDocumentSelected={setDocumentId} />}

      {documentId && !edit && (
        <DocumentViewer
          key={documentId}
          documentId={documentId}
          onBack={() => setDocumentId(undefined)}
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
    </>
  );
}
