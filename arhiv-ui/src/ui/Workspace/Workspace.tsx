import { useEffect, useState } from 'preact/hooks';
import { getQueryParam } from 'utils';
import { useScrollRestoration } from 'utils/hooks';
import { Button } from 'components/Button';
import { DropdownMenu } from 'components/DropdownMenu';
import { Card, throwBadCardVariant, useWorkspaceReducer } from './workspace-reducer';
import { CatalogCard } from './CatalogCard';
import { NewDocumentCard } from './NewDocumentCard';
import { CardContainer } from './CardContainer';
import { DocumentCard } from './DocumentCard';
import { StatusCard } from './StatusCard';
import { FilePickerCard } from './FilePickerCard';
import { ScraperCard } from './ScraperCard';
import { BrowserCard } from './BrowserCard';

export function Workspace() {
  const [wrapperEl, setWrapperEl] = useState<HTMLElement | null>(null);
  useScrollRestoration(wrapperEl, 'workspace-scroll');

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
    <div
      className="w-screen h-full overflow-x-auto pt-12 pb-2 pl-8 pr-16 scroll-smooth"
      ref={setWrapperEl}
    >
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
                icon: 'info',
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

        {cards.map((card) => (
          <CardContainer key={card.id} card={card} dispatch={dispatch}>
            {renderCard(card)}
          </CardContainer>
        ))}
      </div>
    </div>
  );
}

function renderCard(card: Card) {
  switch (card.variant) {
    case 'catalog':
      return <CatalogCard query={card.query} page={card.page} documentType={card.documentType} />;

    case 'browser':
      return <BrowserCard />;

    case 'new-document':
      return <NewDocumentCard documentType={card.documentType} />;

    case 'document':
      return <DocumentCard documentId={card.documentId} />;

    case 'status':
      return <StatusCard />;

    case 'file-picker':
      return <FilePickerCard />;

    case 'scraper':
      return <ScraperCard />;
  }

  throwBadCardVariant(card);
}
