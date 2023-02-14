import { useEffect, useRef, useState } from 'preact/hooks';
import { JSXChildren } from 'utils/jsx';
import { IconButton } from 'components/Button';
import { Card, CardContext, useCardContext, WorkspaceDispatch } from './workspace-reducer';

type CardContainerProps = {
  card: Card;
  dispatch: WorkspaceDispatch;
  children: JSXChildren;
};
export function CardContainer({ card, dispatch, children }: CardContainerProps) {
  const [el, setEl] = useState<HTMLElement | null>(null);

  const cardContextRef = useRef({ card, dispatch });

  useEffect(() => {
    cardContextRef.current = {
      card,
      dispatch,
    };
  }, [card, dispatch]);

  useEffect(() => {
    if (el) {
      delete el.dataset.initializing;

      if (!cardContextRef.current.card.restored) {
        el.scrollIntoView({ inline: 'center' });
      }
    }
  }, [el]);

  return (
    <CardContext.Provider value={cardContextRef.current}>
      <div
        className="var-card-width min-h-[50%] max-h-full overflow-auto shrink-0 grow-0 bg-white drop-shadow relative snap-center transition-opacity data-[initializing]:opacity-30"
        data-initializing
        ref={setEl}
      >
        <div className="px-4 pb-6">{children}</div>
      </div>
    </CardContext.Provider>
  );
}

type TopbarProps = {
  left?: JSXChildren;
  right?: JSXChildren;
  skipBack?: boolean;
};
CardContainer.Topbar = function Topbar({ left, right, skipBack }: TopbarProps) {
  const { hasStackedCards, popStack } = useCardContext();

  return (
    <div className="flex items-center gap-4 bg-white/95 sticky inset-x-0 top-0 z-10 -mx-4 px-4 py-2 mb-4">
      <div className="flex items-center gap-4 justify-start grow">{left}</div>

      <div className="flex items-center gap-1 justify-end grow">
        {hasStackedCards && !skipBack && (
          <IconButton
            icon="arrow-left"
            size="lg"
            title="Go back"
            onClick={popStack}
            className="relative right-2"
          />
        )}
        {right}
      </div>
    </div>
  );
};

CardContainer.CloseButton = function CloseButton() {
  const context = useCardContext();

  const onClose = () => {
    context.close();
  };

  return (
    <IconButton icon="x" size="lg" title="Close" onClick={onClose} className="relative left-1" />
  );
};
