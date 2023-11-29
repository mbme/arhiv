import { useEffect, useState } from 'react';
import { DocumentId } from 'dto';
import { getQueryParam } from 'utils';
import { useScrollRestoration } from 'utils/hooks';
import { Button } from 'components/Button';
import { DropdownMenu } from 'components/DropdownMenu';
import { ScraperDialog } from 'components/ScraperDialog';
import { FilePickerDialog } from 'components/FilePicker/FilePickerDialog';
import { RefClickHandlerContext } from 'components/Ref';
import {
  Card,
  useWorkspaceActions,
  CardContextProvider,
  throwBadCardVariant,
  useWorkspaceReducer,
} from './workspace-reducer';
import { CatalogCard } from './CatalogCard';
import { NewDocumentCard } from './NewDocumentCard';
import { DocumentCardContainer } from './DocumentCard';
import { StatusCard } from './StatusCard';
import { ScrapeResultCard } from './ScrapeResultCard';
import { NewDocumentDialog } from './NewDocumentDialog';
import { ImagePasteHandler } from './ImagePasteHandler';
import { CommitOrSyncButton } from './CommitOrSyncButton';

export function Workspace() {
  const [wrapperEl, setWrapperEl] = useState<HTMLElement | null>(null);
  useScrollRestoration(wrapperEl, 'workspace-scroll');

  const [{ cards }, dispatch] = useWorkspaceReducer();

  const { openDocument, open, closeAll } = useWorkspaceActions(dispatch);

  useEffect(() => {
    const documentId = getQueryParam('id');

    if (documentId) {
      openDocument(documentId as DocumentId, true);
    }
  }, [openDocument]);

  const [showNewDocumentDialog, setShowNewDocumentDialog] = useState(false);
  const [showScraperDialog, setShowScraperDialog] = useState(false);
  const [showFilePickerDialog, setShowFilePickerDialog] = useState(false);

  return (
    <RefClickHandlerContext.Provider value={openDocument}>
      <div
        className="flex flex-row items-start gap-6 h-full overflow-x-auto pt-12 pb-2 px-8 scroll-smooth custom-scrollbar"
        ref={setWrapperEl}
      >
        <nav className="fixed inset-x-0 top-0 z-20 bg-zinc-200 var-bg-color pl-8 pr-4 flex flex-row gap-8">
          <Button variant="text" disabled>
            Player
          </Button>

          <Button
            variant="text"
            leadingIcon="add-document"
            onClick={() => setShowNewDocumentDialog(true)}
            className="ml-auto"
          >
            New...
          </Button>
          {showNewDocumentDialog && (
            <NewDocumentDialog
              onNewDocument={(documentType) => {
                open({ variant: 'new-document', documentType });
                setShowNewDocumentDialog(false);
              }}
              onScrape={() => {
                setShowScraperDialog(true);
                setShowNewDocumentDialog(false);
              }}
              onAttach={() => {
                setShowFilePickerDialog(true);
                setShowNewDocumentDialog(false);
              }}
              onCancel={() => {
                setShowNewDocumentDialog(false);
              }}
            />
          )}

          <Button
            variant="text"
            leadingIcon="search-catalog"
            onClick={() => open({ variant: 'catalog' })}
          >
            Search
          </Button>

          <CommitOrSyncButton />

          {showScraperDialog && (
            <ScraperDialog
              onSuccess={(url, ids) => {
                open({ variant: 'scrape-result', url, ids });
                setShowScraperDialog(false);
              }}
              onCancel={() => {
                setShowScraperDialog(false);
              }}
            />
          )}

          {showFilePickerDialog && (
            <FilePickerDialog
              onAttachmentCreated={(documentId) => {
                open({ variant: 'document', documentId });
                setShowFilePickerDialog(false);
              }}
              onCancel={() => {
                setShowFilePickerDialog(false);
              }}
            />
          )}

          <ImagePasteHandler
            onSuccess={(documentId) => {
              open({ variant: 'document', documentId });
            }}
          />

          <DropdownMenu
            align="bottom-right"
            options={[
              {
                text: 'Status',
                icon: 'info',
                onClick: () => open({ variant: 'status' }),
              },

              process.env.NODE_ENV === 'development' && {
                text: 'Components Demo',
                onClick: () => {
                  window.location.search = 'DEMO';
                },
              },

              {
                text: 'Close cards',
                icon: 'x',
                onClick: closeAll,
              },
            ]}
          />
        </nav>

        {cards.map((card) => (
          <CardContextProvider key={card.id} card={card} dispatch={dispatch}>
            {renderCard(card)}
          </CardContextProvider>
        ))}
      </div>
    </RefClickHandlerContext.Provider>
  );
}

function renderCard(card: Card) {
  switch (card.variant) {
    case 'catalog':
      return <CatalogCard query={card.query} page={card.page} documentType={card.documentType} />;

    case 'new-document':
      return (
        <NewDocumentCard documentType={card.documentType} subtype={card.subtype} data={card.data} />
      );

    case 'document':
      return <DocumentCardContainer documentId={card.documentId} />;

    case 'status':
      return <StatusCard />;

    case 'scrape-result':
      return <ScrapeResultCard url={card.url} ids={card.ids} />;
  }

  throwBadCardVariant(card);
}
