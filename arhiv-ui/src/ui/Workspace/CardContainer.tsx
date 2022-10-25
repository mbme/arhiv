import { useEffect, useRef } from 'preact/hooks';
import { JSXChildren } from '../utils/jsx';
import { IconButton } from '../components/Button';
import { Card, CardContext, useCardContext, WorkspaceDispatch } from './workspace-reducer';

type CardContainerProps = {
  card: Card;
  dispatch: WorkspaceDispatch;
  children: JSXChildren;
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
        className="var-card-width min-h-[50%] max-h-full overflow-auto shrink-0 grow-0 bg-white drop-shadow relative snap-center transition-opacity opacity-30"
        ref={(containerEl) => {
          if (!containerEl || scrolledRef.current) {
            return;
          }

          containerEl.scrollIntoView({ inline: 'center' });
          containerEl.classList.remove('opacity-30');
          scrolledRef.current = true;
        }}
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
    <div className="flex items-center gap-4 bg-white py-2 sticky inset-x-0 top-0 z-10">
      <div className="flex items-center gap-4 justify-start grow">
        {hasStackedCards && !skipBack && (
          <IconButton icon="arrow-left" onClick={popStack} className="relative right-2" />
        )}
        {left}
      </div>

      <div className="flex items-center gap-4 justify-end grow">{right}</div>
    </div>
  );
};

CardContainer.CloseButton = function CloseButton() {
  const context = useCardContext();

  const onClose = () => {
    context.close();
  };

  return <IconButton icon="x" onClick={onClose} className="relative left-3" />;
};
