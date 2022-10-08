import { useCardContext } from '../../workspace-reducer';
import { Catalog } from '../Catalog/Catalog';

type Props = {
  collectionId: string;
  query?: string;
  page?: number;
};
export function CollectionCatalog({ collectionId, query, page }: Props) {
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
    <Catalog
      collectionId={collectionId}
      initialQuery={query}
      initialPage={page}
      onQueryChange={updateQuery}
      onPageChange={updatePage}
      onDocumentSelected={updateDocumentId}
    />
  );
}
