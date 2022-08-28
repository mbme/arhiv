import { createContext } from 'preact';
import { useContext, useEffect, useReducer } from 'preact/hooks';
import { newId, getSessionValue, setSessionValue } from '../scripts/utils';

type CardVariant =
  | { variant: 'catalog' } //
  | { variant: 'file-picker' }
  | { variant: 'status' }
  | { variant: 'new-document'; documentType?: string }
  | { variant: 'document'; documentId: string };

export type Card = CardVariant & { id: number };

export function throwBadCardVariant(value: never): never;
export function throwBadCardVariant(value: CardVariant) {
  throw new Error(`Unknown CardVariant: ${value.variant}`);
}

function createCard(variant: CardVariant): Card {
  return {
    id: newId(),
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
    };

export type WorkspaceDispatch = (action: ActionType) => void;

function workspaceReducer(state: Card[], action: ActionType): Card[] {
  switch (action.type) {
    case 'open': {
      return [...state, createCard(action.newCard)];
    }
    case 'close': {
      return state.filter((card) => card.id !== action.id);
    }
    case 'replace': {
      return state.map((card) => (card.id === action.id ? createCard(action.newCard) : card));
    }
    default: {
      return state;
    }
  }
}

const SESSION_STORAGE_KEY = 'workspace-state';

export function useWorkspaceReducer(): [Card[], WorkspaceDispatch] {
  const [cards, dispatch] = useReducer(workspaceReducer, undefined, () =>
    getSessionValue<Card[]>(SESSION_STORAGE_KEY, []).map((card) => ({
      ...card,
      // override card ids to prevent id clashes after page reload
      id: newId(),
    }))
  );

  useEffect(() => {
    setSessionValue(SESSION_STORAGE_KEY, cards);
  }, [cards]);

  return [cards, dispatch];
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
    close() {
      dispatch({
        type: 'close',
        id: card.id,
      });
    },
    replace(newCard: CardVariant) {
      dispatch({
        type: 'replace',
        id: card.id,
        newCard,
      });
    },
    open(newCard: CardVariant) {
      dispatch({
        type: 'open',
        newCard,
      });
    },
  };
}
