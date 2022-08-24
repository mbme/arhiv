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

  const scrolledRef = useRef(false);

  return (
    <CardContext.Provider value={cardContextRef.current}>
      <div
        className="w-[38rem] shrink-0 grow-0 bg-white drop-shadow relative       snap-center"
        ref={(containerEl) => {
          if (!containerEl || scrolledRef.current) {
            return;
          }

          containerEl.scrollIntoView({ inline: 'center' });
          scrolledRef.current = true;
        }}
      >
        <div className="max-h-full overflow-auto px-4 pt-16 pb-2">{children}</div>
      </div>
    </CardContext.Provider>
  );
}

type TopbarProps = {
  title?: string;
  left?: ComponentChildren;
  right?: ComponentChildren;
};
CardContainer.Topbar = function Topbar({ title, left, right }: TopbarProps) {
  return (
    <div className="flex items-center gap-4 justify-between bg-white px-4 py-2 absolute inset-x-0 top-0 z-10">
      <div className="flex items-center gap-4">{left}</div>

      {title && <div className="section-heading text-lg">{title}</div>}

      <div className="flex items-center gap-4">{right}</div>
    </div>
  );
};

CardContainer.CloseButton = function CloseButton() {
  const context = useCardContext();

  const onClose = () => {
    context.close();
  };

  return (
    <Button variant="link" onClick={onClose}>
      <Icon variant="x" />
    </Button>
  );
};
