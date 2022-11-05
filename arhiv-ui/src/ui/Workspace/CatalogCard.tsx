import { Catalog } from 'components/Catalog/Catalog';
import { useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';

type Props = {
  query?: string;
  page?: number;
  documentType?: string;
};
export function CatalogCard({ query, page, documentType }: Props) {
  const context = useCardContext();

  const updateQuery = (query: string) => {
    context.update({ query });
  };

  const updatePage = (page: number) => {
    context.update({ page });
  };

  const updateDocumentId = (documentId: string) => {
    context.pushStack({ variant: 'document', documentId });
  };

  return (
    <>
      <CardContainer.Topbar
        left={
          <span className="section-heading text-lg">
            {documentType === undefined ? 'Catalog' : `Catalog: ${documentType || 'ERASED'}`}
          </span>
        }
        right={<CardContainer.CloseButton />}
      />

      <Catalog
        autofocus={!context.restored}
        documentType={documentType}
        initialQuery={query}
        initialPage={page}
        onQueryChange={updateQuery}
        onPageChange={updatePage}
        onDocumentSelected={updateDocumentId}
      />
    </>
  );
}
