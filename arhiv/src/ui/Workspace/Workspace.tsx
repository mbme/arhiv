import { useEffect, useState } from 'react';
import { DocumentId } from 'dto';
import { getQueryParam } from 'utils';
import { useScrollRestoration, useSignal } from 'utils/hooks';
import { SuspenseCacheProvider } from 'components/SuspenseCacheProvider';
import { RefClickHandlerContext } from 'components/Ref';
import { Toaster } from 'components/Toaster';
import { ErrorBoundary } from 'components/ErrorBoundary';
import { QueryError } from 'components/QueryError';
import { Card, CardContextProvider, throwBadCardVariant } from './controller';
import { CatalogCard } from './CatalogCard';
import { NewDocumentCard } from './NewDocumentCard';
import { DocumentCardContainer } from './DocumentCard';
import { StatusCard } from './StatusCard';
import { ScrapeResultCard } from './ScrapeResultCard';
import { WorkspaceHeader } from './WorkspaceHeader';
import { CardContainer } from './CardContainer';

export function Workspace() {
  const [wrapperEl, setWrapperEl] = useState<HTMLElement | null>(null);
  useScrollRestoration(wrapperEl, 'workspace-scroll');

  const controller = window.WORKSPACE;
  const cards = useSignal(controller.$cards);

  useEffect(() => {
    const documentId = getQueryParam('id');

    if (documentId) {
      controller.openDocument(documentId as DocumentId, true);
    }
  }, [controller]);

  return (
    <RefClickHandlerContext.Provider value={controller.openDocument}>
      <div className="workspace-cards" ref={setWrapperEl}>
        <WorkspaceHeader controller={controller} />

        {cards.map((card) => (
          <CardContextProvider key={card.id} card={card} controller={controller}>
            <SuspenseCacheProvider cacheId={card.id}>
              <ErrorBoundary renderError={renderError}>{renderCard(card)}</ErrorBoundary>
            </SuspenseCacheProvider>
          </CardContextProvider>
        ))}
      </div>

      <Toaster />
    </RefClickHandlerContext.Provider>
  );
}

function renderCard(card: Card) {
  switch (card.variant) {
    case 'catalog':
      return <CatalogCard />;

    case 'new-document':
      return <NewDocumentCard />;

    case 'document':
      return <DocumentCardContainer />;

    case 'status':
      return <StatusCard />;

    case 'scrape-result':
      return <ScrapeResultCard url={card.url} ids={card.ids} />;
  }

  throwBadCardVariant(card);
}

const renderError = (error: unknown) => (
  <CardContainer>
    <QueryError error={error} />
  </CardContainer>
);
