import { useEffect, useState } from 'react';
import { JSXChildren } from 'utils/jsx';
import { useScrollRestoration } from 'utils/hooks';
import { IconButton } from 'components/Button';
import { Icon } from 'components/Icon';
import { SuspenseBoundary } from 'components/SuspenseBoundary';
import { useCardContext } from './workspace-reducer';

type CardContainerProps = {
  children: JSXChildren;
};
export function CardContainer({ children }: CardContainerProps) {
  const { id, restored } = useCardContext();

  const [el, setEl] = useState<HTMLElement | null>(null);

  useEffect(() => {
    if (el && !restored) {
      el.scrollIntoView({ inline: 'center' });
    }
  }, [el, restored]);

  useScrollRestoration(el, `workspace-card-${id}`);

  const fallback = (
    <div className="card-container flex items-center justify-center">
      <Icon variant="spinner" className="h-10 w-10 opacity-50" />
    </div>
  );

  return (
    <SuspenseBoundary fallback={fallback}>
      <div className="card-container" ref={setEl}>
        <div className="px-4 pb-6 relative">{children}</div>
      </div>
    </SuspenseBoundary>
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
