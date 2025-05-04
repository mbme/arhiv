import { Suspense, useEffect, useRef, useState } from 'react';
import { cx } from 'utils';
import { JSXChildren } from 'utils/jsx';
import { useScrollHandler, useScrollRestoration } from 'utils/hooks';
import { IconButton } from 'components/Button';
import { Icon } from 'components/Icon';
import { FORM_VIEWPORT_CLASSNAME } from 'components/Form/Form';
import { useCardContext } from './controller';

type CardContainerProps = {
  children: JSXChildren;
  leftToolbar?: JSXChildren;
  rightToolbar?: JSXChildren;
  title?: JSXChildren;
  showTitleOnScroll?: boolean;
  skipBack?: boolean;
  skipClose?: boolean;
  className?: string;
  toolbarClassName?: string;
};
export function CardContainer({
  children,
  leftToolbar,
  rightToolbar,
  title,
  showTitleOnScroll = false,
  skipBack,
  skipClose,
  className,
  toolbarClassName,
}: CardContainerProps) {
  const { card, controller } = useCardContext();

  const restored = card.restored;
  const openTime = card.openTime;
  const hasStackedCards = Boolean(card.previousCard);

  const [el, setEl] = useState<HTMLElement | null>(null);

  useScrollRestoration(el, `workspace-card-${card.id}-scroll`);

  const isFirstRef = useRef(true);
  useEffect(() => {
    if (!el) {
      return;
    }

    const isFirstUpdate = isFirstRef.current;
    isFirstRef.current = false;

    if (isFirstUpdate && restored) {
      return;
    }

    el.scrollIntoView({ inline: 'center' });
  }, [el, restored, openTime]);

  const [showTitle, setShowTitle] = useState(showTitleOnScroll ? false : true);
  useScrollHandler(el, (_scrollX, scrollY) => {
    setShowTitle(showTitleOnScroll ? scrollY > 50 : true);
  });

  const fallback = (
    <div className="card-content flex items-center justify-center grow">
      <Icon variant="spinner" className="h-10 w-10 opacity-50" />
    </div>
  );

  return (
    <div className="card-container flex flex-col">
      <div className={cx('card-toolbar', toolbarClassName)}>
        <div className="card-toolbar-left">{leftToolbar}</div>
        <div className="card-toolbar-title">{showTitle && title}</div>
        <div className="card-toolbar-right">
          {hasStackedCards && !skipBack && (
            <IconButton
              icon="arrow-left"
              size="lg"
              title="Go back"
              onClick={() => {
                controller.popStack(card.id);
              }}
              className="relative right-2"
            />
          )}

          {rightToolbar}

          {!skipClose && (
            <IconButton
              icon="x"
              size="lg"
              title="Close"
              onClick={() => {
                controller.close(card.id);
              }}
              className="relative left-1"
            />
          )}
        </div>
      </div>

      <Suspense fallback={fallback}>
        <div className={cx('card-content', FORM_VIEWPORT_CLASSNAME, className)} ref={setEl}>
          {children}
        </div>
      </Suspense>
    </div>
  );
}
