import { useEffect, useState } from 'react';
import { cx } from 'utils';
import { JSXChildren } from 'utils/jsx';
import { useScrollRestoration } from 'utils/hooks';
import { IconButton } from 'components/Button';
import { Icon } from 'components/Icon';
import { SuspenseBoundary } from 'components/SuspenseBoundary';
import { FORM_VIEWPORT_CLASSNAME } from 'components/Form/Form';
import { useCardContext } from './workspace-reducer';

type CardContainerProps = {
  children: JSXChildren;
  leftToolbar?: JSXChildren;
  rightToolbar?: JSXChildren;
  skipBack?: boolean;
  skipClose?: boolean;
};
export function CardContainer({
  children,
  leftToolbar,
  rightToolbar,
  skipBack,
  skipClose,
}: CardContainerProps) {
  const { card, actions } = useCardContext();

  const restored = card.restored;
  const hasStackedCards = Boolean(card.previousCard);

  const [el, setEl] = useState<HTMLElement | null>(null);

  useEffect(() => {
    if (el && !restored) {
      el.scrollIntoView({ inline: 'center' });
    }
  }, [el, restored]);

  useScrollRestoration(el, `workspace-card-${card.id}`);

  const fallback = (
    <div className="card-container flex items-center justify-center">
      <Icon variant="spinner" className="h-10 w-10 opacity-50" />
    </div>
  );

  return (
    <div className="card-container flex flex-col">
      <div className="card-toolbar">
        <div className="card-toolbar-left">{leftToolbar}</div>
        <div className="card-toolbar-right">
          {hasStackedCards && !skipBack && (
            <IconButton
              icon="arrow-left"
              size="lg"
              title="Go back"
              onClick={() => actions.popStack(card.id)}
              className="relative right-2"
            />
          )}

          {rightToolbar}

          {!skipClose && (
            <IconButton
              icon="x"
              size="lg"
              title="Close"
              onClick={() => actions.close(card.id)}
              className="relative left-1"
            />
          )}
        </div>
      </div>

      <div className={cx('card-content', FORM_VIEWPORT_CLASSNAME)} ref={setEl}>
        <SuspenseBoundary fallback={fallback}>{children}</SuspenseBoundary>
      </div>
    </div>
  );
}
