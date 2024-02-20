import { DocumentId } from 'dto';
import { Catalog } from 'components/Catalog/Catalog';
import { Card, useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';

type CatalogCard = Extract<Card, { variant: 'catalog' }>;

export function CatalogCard() {
  const { card, actions } = useCardContext<CatalogCard>();

  const openDocument = (documentId: DocumentId) => {
    actions.pushDocument(card.id, documentId);
  };

  return (
    <CardContainer
      className="pb-0"
      leftToolbar={<span className="section-heading text-lg">Catalog</span>}
    >
      <Catalog
        autofocus={!card.restored}
        query={card.query ?? ''}
        onQueryChange={(query) => actions.update(card.id, { query })}
        page={card.page ?? 0}
        onPageChange={(page) => actions.update(card.id, { page })}
        showSettings={card.showSettings ?? false}
        onToggleSettings={(showSettings) => actions.update(card.id, { showSettings })}
        documentTypes={card.documentTypes ?? []}
        onIncludedDocumentTypesChange={(documentTypes) =>
          actions.update(card.id, { documentTypes })
        }
        onDocumentSelected={openDocument}
      />
    </CardContainer>
  );
}
