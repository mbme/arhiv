import { DocumentId } from 'dto';
import { Catalog } from 'components/Catalog/Catalog';
import { useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';

type Props = {
  query?: string;
  page?: number;
};
export function CatalogCard({ query, page }: Props) {
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
    <CardContainer leftToolbar={<span className="section-heading text-lg">Catalog</span>}>
      <Catalog
        autofocus={!card.restored}
        initialQuery={query}
        initialPage={page}
        onQueryChange={updateQuery}
        onPageChange={updatePage}
        onDocumentSelected={openDocument}
      />
    </CardContainer>
  );
}
