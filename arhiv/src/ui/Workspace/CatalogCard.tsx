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
  const { card, actions } = useCardContext();

  const updateQuery = (query: string) => {
    actions.update(card.id, { query });
  };

  const updatePage = (page: number) => {
    actions.update(card.id, { page });
  };

  const openDocument = (documentId: DocumentId) => {
    actions.pushDocument(card.id, documentId);
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
        autofocus={!card.restored}
        documentTypes={documentType === undefined ? undefined : [documentType]}
        initialQuery={query}
        initialPage={page}
        onQueryChange={updateQuery}
        onPageChange={updatePage}
        onDocumentSelected={openDocument}
      />
    </CardContainer>
  );
}
