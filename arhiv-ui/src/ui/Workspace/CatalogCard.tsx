import { startTransition } from 'react';
import { DocumentId, DocumentType } from 'dto';
import { Catalog } from 'components/Catalog/Catalog';
import { useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';

type Props = {
  query?: string;
  page?: number;
  documentType?: DocumentType;
};
export function CatalogCard({ query, page, documentType }: Props) {
  const context = useCardContext();

  const updateQuery = (query: string) => {
    context.update({ query });
  };

  const updatePage = (page: number) => {
    context.update({ page });
  };

  const updateDocumentId = (documentId: DocumentId) => {
    startTransition(() => {
      context.pushStack({ variant: 'document', documentId });
    });
  };

  return (
    <CardContainer
      leftToolbar={
        <span className="section-heading text-lg">
          {documentType === undefined ? 'Catalog' : `Catalog: ${documentType || 'ERASED'}`}
        </span>
      }
    >
      <Catalog
        autofocus={!context.restored}
        documentTypes={documentType === undefined ? undefined : [documentType]}
        initialQuery={query}
        initialPage={page}
        onQueryChange={updateQuery}
        onPageChange={updatePage}
        onDocumentSelected={updateDocumentId}
      />
    </CardContainer>
  );
}
