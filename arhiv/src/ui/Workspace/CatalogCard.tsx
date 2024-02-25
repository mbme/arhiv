import { NOTE_DOCUMENT_TYPE } from 'dto';
import { Catalog } from 'components/Catalog/Catalog';
import { Card, useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';

type CatalogCard = Extract<Card, { variant: 'catalog' }>;

export function CatalogCard() {
  const { card, actions } = useCardContext<CatalogCard>();

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
        onDocumentSelected={(info) => {
          actions.pushDocument(card.id, info.id);
        }}
        onCreateNote={(title) => {
          actions.open({
            variant: 'new-document',
            documentType: NOTE_DOCUMENT_TYPE,
            data: { title },
          });
        }}
      />
    </CardContainer>
  );
}
