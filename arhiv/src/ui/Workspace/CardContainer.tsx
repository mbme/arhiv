import { useState } from 'react';
import { cx } from 'utils';
import { JSXChildren } from 'utils/jsx';
import { useScrollRestoration } from 'utils/hooks';
import { IconButton } from 'components/Button';
import { Icon } from 'components/Icon';
import { SuspenseBoundary } from 'components/SuspenseBoundary';
import { FORM_VIEWPORT_CLASSNAME } from 'components/Form/Form';
import { QueryError } from 'components/QueryError';
import { useCardContext } from './workspace-reducer';

const renderError = (error: unknown) => (
  <div className="card-container">
    <QueryError error={error} />
  </div>
);

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

  useScrollRestoration(el, `workspace-card-${card.id}`, () => {
    if (el && !restored) {
      el.scrollIntoView({ inline: 'center' });
    }
  });

  const fallback = (
    <div className="card-container flex items-center justify-center grow">
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

      <SuspenseBoundary fallback={fallback} renderError={renderError}>
        <div className={cx('card-content', FORM_VIEWPORT_CLASSNAME)} ref={setEl}>
          {children}
        </div>
      </SuspenseBoundary>
    </div>
  );
}
