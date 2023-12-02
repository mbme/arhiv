import { DocumentId } from 'dto';
import { Catalog } from 'components/Catalog/Catalog';
import { Card, CatalogCardProps, useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';

type CatalogCard = Extract<Card, { variant: 'catalog' }>;

export function CatalogCard() {
  const { card, actions } = useCardContext<CatalogCard>();

  const updateProps = (props: CatalogCardProps) => {
    actions.update(card.id, props);
  };

  const openDocument = (documentId: DocumentId) => {
    actions.pushDocument(card.id, documentId);
  };

  return (
    <CardContainer leftToolbar={<span className="section-heading text-lg">Catalog</span>}>
      <Catalog
        autofocus={!card.restored}
        initialQuery={card.query}
        initialPage={card.page}
        onPropChange={updateProps}
        onDocumentSelected={openDocument}
      />
    </CardContainer>
  );
}
