import { useEffect, createContext, useContext, startTransition } from 'react';
import { effect, signal } from '@preact/signals-core';
import { newId } from 'utils';
import { DocumentData, DocumentId, DocumentType } from 'dto';
import { JSXChildren } from 'utils/jsx';
import { useShallowMemo } from 'utils/hooks';
import { storage } from 'utils/storage';

type CardVariant =
  | {
      variant: 'catalog';
      query?: string;
      page?: number;
      showSettings?: boolean;
      documentTypes?: DocumentType[];
    }
  | {
      variant: 'status';
    }
  | {
      variant: 'scrape-result';
      url: string;
      ids: DocumentId[];
    }
  | {
      variant: 'new-document';
      documentType: DocumentType;
      data?: DocumentData;
      collections?: DocumentId[];
    }
  | {
      variant: 'document';
      documentId: DocumentId;
      forceEditor?: boolean;
    };

export type CardId = string;

export type Card = CardVariant & {
  id: CardId;
  previousCard?: Card;
  restored?: boolean;
  locked?: boolean;
  openTime: number;
};

export function throwBadCardVariant(value: never): never;
export function throwBadCardVariant(value: CardVariant) {
  throw new Error(`Unknown CardVariant: ${value.variant}`);
}

type UpdateActionProps = Omit<Partial<CardVariant>, 'variant'>;

export class WorkspaceController {
  readonly $cards = signal<Card[]>([]);
  readonly $showSearchDialog = signal<[boolean, string]>([false, '']);

  constructor() {
    this.$cards.value = storage.getValue<Card[]>(STORAGE_KEY, []).map((card) => ({
      ...card,
      restored: true,
    }));

    effect(() => {
      storage.setValue(STORAGE_KEY, this.$cards.value);
    });
  }

  open(newCardVariant: CardVariant) {
    const newCard = {
      ...newCardVariant,
      id: newId(),
      openTime: Date.now(),
    };

    const cards = this.$cards.value;

    startTransition(() => {
      this.$cards.value = [...cards, newCard];
    });
  }

  openDocument = (documentId: DocumentId, skipDocumentIfAlreadyOpen = false) => {
    const existingCard = this.$cards.value.find(
      (card) => card.variant === 'document' && card.documentId === documentId,
    );

    if (existingCard && skipDocumentIfAlreadyOpen) {
      this.update(existingCard.id, {
        openTime: Date.now(),
      });
    } else {
      this.open({
        variant: 'document',
        documentId,
      });
    }
  };

  pushStack(id: CardId, newCardVariant: CardVariant) {
    this.replace(id, newCardVariant, true);
  }

  pushDocument(id: CardId, documentId: DocumentId) {
    this.pushStack(id, {
      variant: 'document',
      documentId,
    });
  }

  popStack(id: CardId) {
    if (!this.cardUpdateConfirmed(id)) {
      return;
    }

    const cards = this.$cards.value;
    const pos = cards.findIndex((card) => card.id === id);
    if (pos === -1) {
      throw new Error(`can't pop: can't find card with id ${id}`);
    }
    const card = cards[pos];

    if (!card.previousCard) {
      throw new Error("can't pop: there is no previousCard");
    }

    const newCards = [...cards];
    newCards[pos] = card.previousCard;

    startTransition(() => {
      this.$cards.value = newCards;
    });
  }

  update(id: CardId, props: UpdateActionProps, confirm = true) {
    if (confirm && !this.cardUpdateConfirmed(id)) {
      return;
    }

    this.$cards.value = this.$cards.value.map((card) => {
      if (card.id === id) {
        return {
          ...card,
          ...props,
        };
      }

      return card;
    });
  }

  lockCard(id: CardId) {
    this.update(id, { locked: true }, false);
  }

  unlockCard(id: CardId) {
    this.update(id, { locked: false }, false);
  }

  close(id: CardId) {
    if (!this.cardUpdateConfirmed(id)) {
      return;
    }

    this.$cards.value = this.$cards.value.filter((card) => card.id !== id);
  }

  closeAll() {
    // keep only locked cards
    this.$cards.value = this.$cards.value.filter((card) => card.locked);
  }

  replace(id: CardId, newCardVariant: CardVariant, stackPrevious = false) {
    if (!this.cardUpdateConfirmed(id)) {
      return;
    }

    const newCard = {
      ...newCardVariant,
      id: newId(),
      openTime: Date.now(),
    };

    const cards = this.$cards.value;
    const pos = cards.findIndex((card) => card.id === id);
    if (pos === -1) {
      throw new Error(`can't replace card: can't find card with id ${id}`);
    }

    const prevCard = stackPrevious ? cards[pos] : undefined;

    const newCards = [...cards];
    newCards[pos] = {
      previousCard: prevCard,
      ...newCard,
    };

    startTransition(() => {
      this.$cards.value = newCards;
    });
  }

  private cardUpdateConfirmed(id: CardId): boolean {
    const card = this.$cards.value.find((item) => item.id === id);
    if (!card) {
      throw new Error(`Can't find card ${id}`);
    }

    if (!card.locked) {
      return true;
    }

    return window.confirm('The card may contain unsaved changes. Continue?');
  }

  showSearchDialog(query = '') {
    startTransition(() => {
      this.$showSearchDialog.value = [true, query];
    });
  }

  hideSearchDialog() {
    startTransition(() => {
      this.$showSearchDialog.value = [false, ''];
    });
  }
}

const STORAGE_KEY = 'workspace-state';

type CardContextType = {
  card: Card;
  controller: WorkspaceController;
};
const CardContext = createContext<CardContextType | undefined>(undefined);

type CardContextProviderProps = {
  card: Card;
  controller: WorkspaceController;
  children: JSXChildren;
};
export function CardContextProvider({ card, controller, children }: CardContextProviderProps) {
  const value = useShallowMemo({ card, controller });

  return <CardContext.Provider value={value}>{children}</CardContext.Provider>;
}

export function useCardContext<C extends Card = Card>() {
  const context = useContext(CardContext);

  if (!context) {
    throw new Error('CardContext is missing');
  }

  const { card, controller } = context;

  return {
    card: card as C,
    controller,
  };
}

export function useCardLock(lockCard: boolean) {
  const { card, controller } = useCardContext();

  useEffect(() => {
    if (!lockCard) {
      return;
    }

    controller.lockCard(card.id);

    return () => {
      controller.unlockCard(card.id);
    };
  }, [lockCard, card.id, controller]);
}
