import { ComponentChildren } from 'preact';
import { useEffect, useRef } from 'preact/hooks';
import { Card, CardContext, useCardContext, WorkspaceDispatch } from '../workspace-reducer';
import { Button } from './Button';
import { Icon } from './Icon';

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
      <div className="w-[38rem] shrink-0 grow-0 bg-white px-4 py-2 overflow-auto">{children}</div>
    </CardContext.Provider>
  );
}

type TopbarProps = {
  children: ComponentChildren;
};
CardContainer.Topbar = function Topbar({ children }: TopbarProps) {
  return (
    <div className="flex gap-2 justify-end bg-neutral-200 mb-6 sticky top-0 z-10">{children}</div>
  );
};

CardContainer.CloseButton = function CloseButton() {
  const context = useCardContext();

  const onClose = () => {
    context.close();
  };

  return (
    <Button variant="icon" onClick={onClose}>
      <Icon variant="x" />
    </Button>
  );
};
