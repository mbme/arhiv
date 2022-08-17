import { useState } from 'preact/hooks';
import { newId } from '../../scripts/utils';
import { CatalogCard } from './CatalogCard';
import { NewDocumentCard } from './NewDocumentCard';

type CardVariant = 'catalog' | 'new-document';

type Card = {
  id: number;
  variant: CardVariant;
};

export function Workspace() {
  const [cards] = useState<Card[]>([
    { id: newId(), variant: 'new-document' }, //
    { id: newId(), variant: 'catalog' }, //
  ]);

  return (
    <div className="flex flex-row gap-4 h-full w-auto overflow-x-auto p-4">
      {cards.map((card) => {
        switch (card.variant) {
          case 'catalog':
            return <CatalogCard key={card.id} />;

          case 'new-document':
            return <NewDocumentCard key={card.id} />;
        }

        throw new Error('unreachable');
      })}
    </div>
  );
}
