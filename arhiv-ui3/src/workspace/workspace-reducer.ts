import produce from 'immer';
import { createContext } from 'preact';
import { useContext, useEffect, useReducer } from 'preact/hooks';
import { newId, getSessionValue, setSessionValue } from '../scripts/utils';

type CardVariant =
  | { variant: 'catalog'; query?: string; page?: number; documentType?: string } //
  | { variant: 'browser' }
  | { variant: 'file-picker' }
  | { variant: 'status' }
  | { variant: 'scraper' }
  | { variant: 'new-document'; documentType?: string }
  | { variant: 'document'; documentId: string; query?: string; page?: number };

export type Card = CardVariant & { id: number; previousCard?: CardVariant };

export function throwBadCardVariant(value: never): never;
export function throwBadCardVariant(value: CardVariant) {
  throw new Error(`Unknown CardVariant: ${value.variant}`);
}

function createCard(variant: CardVariant, previousCard?: CardVariant): Card {
  return {
    id: newId(),
    previousCard,
    ...variant,
  };
}

type ActionType =
  | {
      type: 'open';
      newCard: CardVariant;
    }
  | {
      type: 'close';
      id: number;
    }
  | {
      type: 'replace';
      id: number;
      newCard: CardVariant;
      stackPrevious?: boolean;
    }
  | {
      type: 'update';
      id: number;
      props: UpdateActionProps;
    }
  | {
      type: 'close-all';
    }
  | {
      type: 'lock-card';
      id: number;
    }
  | {
      type: 'unlock-card';
      id: number;
    };

type UpdateActionProps = Omit<Partial<CardVariant>, 'variant'>;

export type WorkspaceDispatch = (action: ActionType) => void;

type WorkspaceState = {
  cards: Card[];
  lockedCardIds: Set<number>;
};
function workspaceReducer(state: WorkspaceState, action: ActionType): WorkspaceState {
  switch (action.type) {
    case 'open': {
      return produce(state, (newState) => {
        newState.cards.push(createCard(action.newCard));
      });
    }

    case 'close': {
      if (!cardUpdateConfirmed(state, action.id)) {
        return state;
      }

      return produce(state, (newState) => {
        newState.cards = newState.cards.filter((card) => card.id !== action.id);
      });
    }

    case 'replace': {
      if (!cardUpdateConfirmed(state, action.id)) {
        return state;
      }

      return produce(state, (newState) => {
        const pos = newState.cards.findIndex((card) => card.id === action.id);
        if (pos > -1) {
          const prevCard = action.stackPrevious ? newState.cards[pos] : undefined;
          newState.cards[pos] = createCard(action.newCard, prevCard);
        }
      });
    }

    case 'update': {
      if (!cardUpdateConfirmed(state, action.id)) {
        return state;
      }

      return produce(state, (newState) => {
        const card = newState.cards.find((card) => card.id === action.id);
        if (!card) {
          throw new Error("can't find card by id");
        }

        Object.assign(card, action.props);
      });
    }

    case 'close-all': {
      return produce(state, (newState) => {
        // keep only locked cards
        newState.cards = newState.cards.filter((card) => newState.lockedCardIds.has(card.id));
      });
    }

    case 'lock-card': {
      return produce(state, (newState) => {
        newState.lockedCardIds.add(action.id);
      });
    }

    case 'unlock-card': {
      return produce(state, (newState) => {
        newState.lockedCardIds.delete(action.id);
      });
    }

    default: {
      return state;
    }
  }
}

function cardUpdateConfirmed(state: WorkspaceState, id: number) {
  return (
    !state.lockedCardIds.has(id) ||
    window.confirm('The card may contain unsaved changes. Continue?')
  );
}

const SESSION_STORAGE_KEY = 'workspace-state';

export function useWorkspaceReducer(): [WorkspaceState, WorkspaceDispatch] {
  const [state, dispatch] = useReducer(workspaceReducer, undefined, () => {
    const cards = getSessionValue<Card[]>(SESSION_STORAGE_KEY, []).map((card) => ({
      ...card,
      // override card ids to prevent id clashes after page reload
      id: newId(),
    }));

    return {
      cards,
      lockedCardIds: new Set([]),
    };
  });

  useEffect(() => {
    setSessionValue(SESSION_STORAGE_KEY, state.cards);
  }, [state.cards]);

  return [state, dispatch];
}

type CardContextType = {
  card: Card;
  dispatch: WorkspaceDispatch;
};
export const CardContext = createContext<CardContextType | undefined>(undefined);

export function useCardContext() {
  const context = useContext(CardContext);

  if (!context) {
    throw new Error('CardContext is missing');
  }

  const { card, dispatch } = context;

  return {
    close: () => {
      dispatch({
        type: 'close',
        id: card.id,
      });
    },
    replace: (newCard: CardVariant) => {
      dispatch({
        type: 'replace',
        id: card.id,
        newCard,
      });
    },
    hasStackedCards: Boolean(card.previousCard),
    pushStack: (newCard: CardVariant) => {
      dispatch({
        type: 'replace',
        id: card.id,
        newCard,
        stackPrevious: true,
      });
    },
    popStack: () => {
      const newCard = card.previousCard;
      if (!newCard) {
        throw new Error("can't pop: there is no previousCard");
      }

      dispatch({
        type: 'replace',
        id: card.id,
        newCard,
      });
    },
    update: (props: UpdateActionProps) => {
      dispatch({
        type: 'update',
        id: card.id,
        props,
      });
    },
    open: (newCard: CardVariant) => {
      dispatch({
        type: 'open',
        newCard,
      });
    },

    lock: () => {
      dispatch({
        type: 'lock-card',
        id: card.id,
      });
    },

    unlock: () => {
      dispatch({
        type: 'unlock-card',
        id: card.id,
      });
    },
  };
}

export function useCardLock() {
  const context = useCardContext();

  useEffect(() => {
    context.lock();

    return () => {
      context.unlock();
    };
  }, []);
}
