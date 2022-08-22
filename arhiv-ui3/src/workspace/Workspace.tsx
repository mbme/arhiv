import { useEffect, useReducer } from 'preact/hooks';
import { throwBadCardVariant, workspaceReducer } from './workspace-reducer';
import { CatalogCard } from './components/CatalogCard';
import { NewDocumentCard } from './components/NewDocumentCard';
import { CardContainer } from './components/CardContainer';
import { DocumentCard } from './components/DocumentCard';

export function Workspace() {
  const [cards, dispatch] = useReducer(workspaceReducer, []);

  useEffect(() => {
    dispatch({ type: 'open', newCard: { variant: 'new-document', documentType: 'note' } });
    dispatch({ type: 'open', newCard: { variant: 'catalog' } });
  }, []);

  return (
    <div className="flex flex-row gap-4 h-full w-auto overflow-x-auto p-4">
      {cards.map((card) => {
        switch (card.variant) {
          case 'catalog':
            return (
              <CardContainer key={card.id} card={card} dispatch={dispatch}>
                <CatalogCard />
              </CardContainer>
            );

          case 'new-document':
            return (
              <CardContainer key={card.id} card={card} dispatch={dispatch}>
                <NewDocumentCard documentType={card.documentType} />
              </CardContainer>
            );

          case 'document':
            return (
              <CardContainer key={card.id} card={card} dispatch={dispatch}>
                <DocumentCard documentId={card.documentId} />
              </CardContainer>
            );
        }

        throwBadCardVariant(card);
      })}
    </div>
  );
}
