import {
  useEffect,
  useReducer,
  createContext,
  startTransition,
  useRef,
  useContext,
  useMemo,
} from 'react';
import { newId, getSessionValue, setSessionValue } from 'utils';
import { DocumentData, DocumentId, DocumentSubtype, DocumentType } from 'dto';
import { JSXChildren } from 'utils/jsx';

type CardVariant =
  | {
      variant: 'catalog';
      query?: string;
      page?: number;
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
      subtype?: DocumentSubtype;
      data?: DocumentData;
    }
  | {
      variant: 'document';
      documentId: DocumentId;
    };

export type CardId = string;

export type Card = CardVariant & {
  id: CardId;
  previousCard?: CardVariant;
  restored?: boolean;
  locked?: boolean;
};

export function throwBadCardVariant(value: never): never;
export function throwBadCardVariant(value: CardVariant) {
  throw new Error(`Unknown CardVariant: ${value.variant}`);
}

function createCard(variant: CardVariant, previousCard?: CardVariant): Card {
  return {
    id: newId(),
    previousCard,
    restored: false,
    ...variant,
  };
}

type ActionType =
  | {
      type: 'open';
      newCard: CardVariant;
      skipDocumentIfAlreadyOpen?: boolean;
    }
  | {
      type: 'close';
      id: CardId;
    }
  | {
      type: 'replace';
      id: CardId;
      newCard: CardVariant;
      stackPrevious?: boolean;
    }
  | {
      type: 'pop';
      id: CardId;
    }
  | {
      type: 'update';
      id: CardId;
      props: UpdateActionProps;
    }
  | {
      type: 'close-all';
    }
  | {
      type: 'lock-card';
      id: CardId;
    }
  | {
      type: 'unlock-card';
      id: CardId;
    };

type UpdateActionProps = Omit<Partial<CardVariant>, 'variant'>;

export type WorkspaceDispatch = (action: ActionType) => void;

type WorkspaceState = {
  cards: Card[];
};
function workspaceReducer(state: WorkspaceState, action: ActionType): WorkspaceState {
  switch (action.type) {
    case 'open': {
      if (action.skipDocumentIfAlreadyOpen && action.newCard.variant === 'document') {
        const { documentId } = action.newCard;

        const isAlreadyOpen = state.cards.some(
          (card) => card.variant === 'document' && card.documentId === documentId,
        );

        if (isAlreadyOpen) {
          return state;
        }
      }

      return {
        ...state,
        cards: [...state.cards, createCard(action.newCard)],
      };
    }

    case 'close': {
      if (!cardUpdateConfirmed(state, action.id)) {
        return state;
      }

      return {
        ...state,
        cards: state.cards.filter((card) => card.id !== action.id),
      };
    }

    case 'replace': {
      if (!cardUpdateConfirmed(state, action.id)) {
        return state;
      }

      const pos = state.cards.findIndex((card) => card.id === action.id);
      if (pos === -1) {
        throw new Error(`can't replace card: can't find card with id ${action.id}`);
      }

      const prevCard = action.stackPrevious ? state.cards[pos] : undefined;

      const newCards = [...state.cards];
      newCards[pos] = createCard(action.newCard, prevCard);

      return {
        ...state,
        cards: newCards,
      };
    }

    case 'pop': {
      if (!cardUpdateConfirmed(state, action.id)) {
        return state;
      }

      const pos = state.cards.findIndex((card) => card.id === action.id);
      if (pos === -1) {
        throw new Error(`can't pop: can't find card with id ${action.id}`);
      }
      const card = state.cards[pos];

      if (!card.previousCard) {
        throw new Error("can't pop: there is no previousCard");
      }

      const newCards = [...state.cards];
      newCards[pos] = createCard(card.previousCard);

      return {
        ...state,
        cards: newCards,
      };
    }

    case 'update': {
      if (!cardUpdateConfirmed(state, action.id)) {
        return state;
      }

      return {
        ...state,
        cards: state.cards.map((card) => {
          if (card.id === action.id) {
            return {
              ...card,
              ...action.props,
            };
          }

          return card;
        }),
      };
    }

    case 'close-all': {
      return {
        ...state,
        // keep only locked cards
        cards: state.cards.filter((card) => card.locked),
      };
    }

    case 'lock-card': {
      return {
        ...state,
        cards: state.cards.map((card) => {
          if (card.id === action.id) {
            return {
              ...card,
              locked: true,
            };
          }
          return card;
        }),
      };
    }

    case 'unlock-card': {
      return {
        ...state,
        cards: state.cards.map((card) => {
          if (card.id === action.id) {
            return {
              ...card,
              locked: false,
            };
          }
          return card;
        }),
      };
    }

    default: {
      return state;
    }
  }
}

function cardUpdateConfirmed(state: WorkspaceState, id: CardId) {
  const card = state.cards.find((item) => item.id === id);
  if (!card) {
    throw new Error(`Can't find card ${id}`);
  }

  if (!card.locked) {
    return true;
  }

  return window.confirm('The card may contain unsaved changes. Continue?');
}

const SESSION_STORAGE_KEY = 'workspace-state';

export function useWorkspaceReducer(): [WorkspaceState, WorkspaceDispatch] {
  const [state, dispatch] = useReducer(workspaceReducer, undefined, () => {
    const cards = getSessionValue<Card[]>(SESSION_STORAGE_KEY, []).map((card) => ({
      ...card,
      restored: true,
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

export function useWorkspaceActions(dispatch: WorkspaceDispatch) {
  return useMemo(
    () => ({
      close: (id: CardId) => {
        startTransition(() => {
          dispatch({
            type: 'close',
            id,
          });
        });
      },
      closeAll: () => {
        startTransition(() => {
          dispatch({
            type: 'close-all',
          });
        });
      },
      replace: (id: CardId, newCard: CardVariant) => {
        startTransition(() => {
          dispatch({
            type: 'replace',
            id,
            newCard,
          });
        });
      },
      pushStack: (id: CardId, newCard: CardVariant) => {
        startTransition(() => {
          dispatch({
            type: 'replace',
            id,
            newCard,
            stackPrevious: true,
          });
        });
      },
      popStack: (id: CardId) => {
        startTransition(() => {
          dispatch({
            type: 'pop',
            id,
          });
        });
      },
      pushDocument: (cardId: CardId, documentId: DocumentId) => {
        startTransition(() => {
          dispatch({
            type: 'replace',
            id: cardId,
            newCard: { variant: 'document', documentId },
            stackPrevious: true,
          });
        });
      },
      update: (id: CardId, props: UpdateActionProps) => {
        dispatch({
          type: 'update',
          id,
          props,
        });
      },
      open: (newCard: CardVariant) => {
        startTransition(() => {
          dispatch({
            type: 'open',
            newCard,
          });
        });
      },

      openDocument: (documentId: DocumentId, skipDocumentIfAlreadyOpen = false) => {
        startTransition(() => {
          dispatch({
            type: 'open',
            newCard: { variant: 'document', documentId },
            skipDocumentIfAlreadyOpen,
          });
        });
      },

      lock: (id: CardId) => {
        dispatch({
          type: 'lock-card',
          id,
        });
      },

      unlock: (id: CardId) => {
        dispatch({
          type: 'unlock-card',
          id,
        });
      },
    }),
    [dispatch],
  );
}

type CardContextType = {
  card: Card;
  dispatch: WorkspaceDispatch;
};
const CardContext = createContext<CardContextType | undefined>(undefined);

type CardContextProviderProps = {
  card: Card;
  dispatch: WorkspaceDispatch;
  children: JSXChildren;
};
export function CardContextProvider({ card, dispatch, children }: CardContextProviderProps) {
  const cardContextRef = useRef({ card, dispatch });

  useEffect(() => {
    cardContextRef.current = {
      card,
      dispatch,
    };
  }, [card, dispatch]);

  return <CardContext.Provider value={cardContextRef.current}>{children}</CardContext.Provider>;
}

export function useCardContext() {
  const context = useContext(CardContext);

  if (!context) {
    throw new Error('CardContext is missing');
  }

  const { card, dispatch } = context;

  return {
    card,
    actions: useWorkspaceActions(dispatch),
  };
}

export function useCardLock(lockCard: boolean) {
  const {
    card,
    actions: { lock, unlock },
  } = useCardContext();

  useEffect(() => {
    if (!lockCard) {
      return;
    }

    lock(card.id);

    return () => {
      unlock(card.id);
    };
  }, [lockCard, card.id, lock, unlock]);
}
