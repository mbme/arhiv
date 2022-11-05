import { useEffect } from 'preact/hooks';
import { getQueryParam } from '../utils';
import { Button } from '../components/Button';
import { DropdownMenu } from '../components/DropdownMenu';
import { throwBadCardVariant, useWorkspaceReducer } from './workspace-reducer';
import { CatalogCard } from './CatalogCard';
import { NewDocumentCard } from './NewDocumentCard';
import { CardContainer } from './CardContainer';
import { DocumentCard } from './DocumentCard';
import { StatusCard } from './StatusCard';
import { FilePickerCard } from './FilePickerCard';
import { ScraperCard } from './ScraperCard';
import { BrowserCard } from './BrowserCard';

export function Workspace() {
  const [{ cards }, dispatch] = useWorkspaceReducer();

  useEffect(() => {
    const documentId = getQueryParam('id');

    if (documentId) {
      dispatch({
        type: 'open',
        newCard: { variant: 'document', documentId },
        skipDocumentIfAlreadyOpen: true,
      });
    }
  }, []);

  return (
    <div className="w-screen h-full overflow-x-auto pt-12 pb-2 pl-8 pr-16 scroll-smooth">
      <div className="flex flex-row items-start gap-6 h-full w-fit min-w-full">
        <nav className="fixed inset-x-0 top-0 z-20 bg-zinc-200 var-bg-color pl-8 pr-4 flex flex-row gap-8">
          <Button
            variant="text"
            leadingIcon="search-catalog"
            onClick={() => dispatch({ type: 'open', newCard: { variant: 'catalog' } })}
          >
            Search
          </Button>

          <Button variant="text" disabled>
            Player
          </Button>

          <Button
            variant="text"
            leadingIcon="add-document"
            onClick={() => dispatch({ type: 'open', newCard: { variant: 'new-document' } })}
            className="ml-auto"
          >
            New...
          </Button>

          <Button
            variant="text"
            leadingIcon="paperclip"
            onClick={() => dispatch({ type: 'open', newCard: { variant: 'file-picker' } })}
          >
            Add file
          </Button>

          <DropdownMenu
            options={[
              {
                text: 'Scrape URL',
                icon: 'web',
                onClick: () => dispatch({ type: 'open', newCard: { variant: 'scraper' } }),
              },

              {
                text: 'Browse',
                icon: 'browse-catalog',
                onClick: () => dispatch({ type: 'open', newCard: { variant: 'browser' } }),
              },

              {
                text: 'Status',
                onClick: () => dispatch({ type: 'open', newCard: { variant: 'status' } }),
              },

              {
                text: 'Components Demo',
                onClick: () => {
                  window.location.search = 'DEMO';
                },
              },

              {
                text: 'Close cards',
                icon: 'x',
                onClick: () => {
                  dispatch({
                    type: 'close-all',
                  });
                },
              },
            ]}
          />
        </nav>

        {cards.map((card) => {
          switch (card.variant) {
            case 'catalog':
              return (
                <CardContainer key={card.id} card={card} dispatch={dispatch}>
                  <CatalogCard
                    query={card.query}
                    page={card.page}
                    documentType={card.documentType}
                  />
                </CardContainer>
              );

            case 'browser':
              return (
                <CardContainer key={card.id} card={card} dispatch={dispatch}>
                  <BrowserCard />
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
                  <DocumentCard documentId={card.documentId} query={card.query} page={card.page} />
                </CardContainer>
              );

            case 'status':
              return (
                <CardContainer key={card.id} card={card} dispatch={dispatch}>
                  <StatusCard />
                </CardContainer>
              );

            case 'file-picker':
              return (
                <CardContainer key={card.id} card={card} dispatch={dispatch}>
                  <FilePickerCard />
                </CardContainer>
              );

            case 'scraper':
              return (
                <CardContainer key={card.id} card={card} dispatch={dispatch}>
                  <ScraperCard />
                </CardContainer>
              );
          }

          throwBadCardVariant(card);
        })}
      </div>
    </div>
  );
}
