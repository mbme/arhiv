import { ComponentChildren } from 'preact';
import { useEffect, useRef } from 'preact/hooks';
import { Card, CardContext, WorkspaceDispatch } from '../workspace-reducer';

type CardContainerProps = {
  card: Card;
  dispatch: WorkspaceDispatch;
  children: ComponentChildren;
};
export function CardContainer({ card, dispatch, children }: CardContainerProps) {
  const cardContextRef = useRef({ card, dispatch });

  useEffect(() => {
    cardContextRef.current = {
      card,
      dispatch,
    };
  }, [card, dispatch]);

  return (
    <CardContext.Provider value={cardContextRef.current}>
      <div className="w-[38rem] shrink-0 bg-white px-4 py-2 overflow-auto">{children}</div>
    </CardContext.Provider>
  );
}
