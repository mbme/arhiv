import { useState } from 'preact/hooks';
import { Catalog } from './Catalog';
import { DocumentViewer } from './DocumentViewer/DocumentViewer';

export function Workspace() {
  const [documentId, setDocumentId] = useState<string>();

  return (
    <div>
      <Catalog hidden={Boolean(documentId)} onDocumentSelected={setDocumentId} />

      {documentId && <DocumentViewer key={documentId} documentId={documentId} />}
    </div>
  );
}
