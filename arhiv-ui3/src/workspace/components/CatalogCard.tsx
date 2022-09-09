import { useEffect, useState } from 'preact/hooks';
import { useCardContext } from '../workspace-reducer';
import { Catalog } from './Catalog/Catalog';
import { DocumentEditor } from './DocumentEditor/DocumentEditor';
import { DocumentViewer } from './DocumentViewer/DocumentViewer';

type CatalogCardProps = {
  query: string;
  page: number;
  documentId?: string;
};
export function CatalogCard({ query, page, documentId: initialDocumentId }: CatalogCardProps) {
  const context = useCardContext();

  const [documentId, _setDocumentId] = useState(initialDocumentId);
  const [edit, setEdit] = useState(false);

  useEffect(() => {
    if (!documentId) {
      setEdit(false);
    }
  }, [documentId]);

  const updateQuery = (query: string) => {
    context.update({ query });
  };

  const updatePage = (page: number) => {
    context.update({ page });
  };

  const updateDocumentId = (documentId?: string) => {
    _setDocumentId(documentId);
    context.update({ documentId });
  };

  return (
    <>
      {!documentId && (
        <Catalog
          initialQuery={query}
          initialPage={page}
          onQueryChange={updateQuery}
          onPageChange={updatePage}
          onDocumentSelected={updateDocumentId}
        />
      )}

      {documentId && !edit && (
        <DocumentViewer
          key={documentId}
          documentId={documentId}
          onBack={() => updateDocumentId(undefined)}
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
