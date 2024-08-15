import { NOTE_DOCUMENT_TYPE } from 'dto';
import { Catalog } from 'components/Catalog/Catalog';
import { Card, useCardContext } from './controller';
import { CardContainer } from './CardContainer';

type CatalogCard = Extract<Card, { variant: 'catalog' }>;

export function CatalogCard() {
  const { card, controller } = useCardContext<CatalogCard>();

  return (
    <CardContainer className="pb-0" title="Catalog">
      <Catalog
        autofocus={!card.restored}
        query={card.query ?? ''}
        onQueryChange={(query) => {
          controller.update(card.id, { query });
        }}
        page={card.page ?? 0}
        onPageChange={(page) => {
          controller.update(card.id, { page });
        }}
        showSettings={card.showSettings ?? false}
        onToggleSettings={(showSettings) => {
          controller.update(card.id, { showSettings });
        }}
        documentTypes={card.documentTypes ?? []}
        onIncludedDocumentTypesChange={(documentTypes) => {
          controller.update(card.id, { documentTypes });
        }}
        onDocumentSelected={(info) => {
          controller.pushDocument(card.id, info.id);
        }}
        onCreateNote={(title) => {
          controller.newDocument(NOTE_DOCUMENT_TYPE, { title });
        }}
      />
    </CardContainer>
  );
}
