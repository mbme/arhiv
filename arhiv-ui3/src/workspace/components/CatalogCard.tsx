import { useCardContext } from '../workspace-reducer';
import { CardContainer } from './CardContainer';
import { Catalog } from './Catalog/Catalog';

type CatalogCardProps = {
  query?: string;
  page?: number;
};
export function CatalogCard({ query, page }: CatalogCardProps) {
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
        left={<span className="section-heading text-lg">Catalog</span>}
        right={<CardContainer.CloseButton />}
      />

      <Catalog
        initialQuery={query}
        initialPage={page}
        onQueryChange={updateQuery}
        onPageChange={updatePage}
        onDocumentSelected={updateDocumentId}
      />
    </>
  );
}
