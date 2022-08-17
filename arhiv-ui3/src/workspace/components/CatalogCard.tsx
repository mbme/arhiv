import { useEffect, useState } from 'preact/hooks';
import { Card } from './Card';
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
    <Card>
      {!documentId && <Catalog onDocumentSelected={setDocumentId} />}

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
    </Card>
  );
}
