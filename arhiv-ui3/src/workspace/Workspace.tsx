import { useEffect, useReducer } from 'preact/hooks';
import { throwBadCardVariant, workspaceReducer } from './workspace-reducer';
import { CatalogCard } from './components/CatalogCard';
import { NewDocumentCard } from './components/NewDocumentCard';
import { CardContainer } from './components/CardContainer';
import { DocumentCard } from './components/DocumentCard';
import { StatusCard } from './components/StatusCard';
import { Button } from './components/Button';
import { Icon } from './components/Icon';

export function Workspace() {
  const [cards, dispatch] = useReducer(workspaceReducer, []);

  useEffect(() => {
    dispatch({ type: 'open', newCard: { variant: 'catalog' } });
  }, []);

  return (
    <div className="relative flex flex-row gap-8 h-full w-auto overflow-x-auto pt-14 pb-2 pl-8 pr-16">
      <nav className="fixed inset-x-0 top-0 z-20 bg-zinc-200 var-bg-color px-16 pt-4 flex flex-row gap-8">
        <Button
          variant="text"
          onClick={() => dispatch({ type: 'open', newCard: { variant: 'catalog' } })}
        >
          <Icon variant="search-catalog" className="mr-1" />
          Browse
        </Button>

        <Button
          variant="text"
          onClick={() => dispatch({ type: 'open', newCard: { variant: 'new-document' } })}
        >
          <Icon variant="add-document" className="mr-1" />
          New...
        </Button>

        <Button
          variant="text"
          onClick={() => dispatch({ type: 'open', newCard: { variant: 'status' } })}
        >
          Status
        </Button>

        <Button variant="text">Scrape URL</Button>

        <Button variant="text">Player</Button>
      </nav>

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

          case 'status':
            return (
              <CardContainer key={card.id} card={card} dispatch={dispatch}>
                <StatusCard />
              </CardContainer>
            );
        }

        throwBadCardVariant(card);
      })}
    </div>
  );
}
