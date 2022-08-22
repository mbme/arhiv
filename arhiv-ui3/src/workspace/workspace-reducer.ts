import { createContext } from 'preact';
import { useContext } from 'preact/hooks';
import { newId } from '../scripts/utils';

type CardVariant =
  | { variant: 'catalog' } //
  | { variant: 'new-document'; documentType: string }
  | { variant: 'document'; documentId: string };

export type Card = CardVariant & { id: number };

export function throwBadCardVariant(value: never): never;
export function throwBadCardVariant(value: CardVariant) {
  throw new Error(`Unknown CardVariant: ${value.variant}`);
}

function newCard(variant: CardVariant): Card {
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

export function workspaceReducer(state: Card[], action: ActionType): Card[] {
  switch (action.type) {
    case 'open': {
      return [...state, newCard(action.newCard)];
    }
    case 'close': {
      return state.filter((card) => card.id !== action.id);
    }
    case 'replace': {
      return state.map((card) => (card.id === action.id ? newCard(action.newCard) : card));
    }
    default: {
      return state;
    }
  }
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
  };
}
