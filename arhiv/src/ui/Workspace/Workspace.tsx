import { useEffect, useState } from 'react';
import { DocumentId } from 'dto';
import { getQueryParam } from 'utils';
import { useScrollRestoration } from 'utils/hooks';
import { SuspenseCacheProvider, useSuspenseCacheCleaner } from 'components/SuspenseCacheProvider';
import { RefClickHandlerContext } from 'components/Ref';
import { Toaster } from 'components/Toaster';
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
import { WorkspaceHeader } from './WorkspaceHeader';

export function Workspace() {
  const [wrapperEl, setWrapperEl] = useState<HTMLElement | null>(null);
  useScrollRestoration(wrapperEl, 'workspace-scroll');

  const [{ cards }, dispatch] = useWorkspaceReducer();

  useRemoveUnusedCardCaches(cards);

  const { openDocument } = useWorkspaceActions(dispatch);

  useEffect(() => {
    const documentId = getQueryParam('id');

    if (documentId) {
      openDocument(documentId as DocumentId, true);
    }
  }, [openDocument]);

  return (
    <RefClickHandlerContext.Provider value={openDocument}>
      <div className="workspace-cards" ref={setWrapperEl}>
        <WorkspaceHeader dispatch={dispatch} />

        {cards.map((card) => (
          <CardContextProvider key={card.id} card={card} dispatch={dispatch}>
            <SuspenseCacheProvider cacheId={card.id}>{renderCard(card)}</SuspenseCacheProvider>
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

function collectCardIds(card: Card, collection: string[]) {
  collection.push(card.id);

  if (card.previousCard) {
    collectCardIds(card.previousCard, collection);
  }
}

function useRemoveUnusedCardCaches(cards: Card[]) {
  const ids: string[] = [];
  cards.forEach((card) => collectCardIds(card, ids));
  useSuspenseCacheCleaner(ids);
}
