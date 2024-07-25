import { forwardRef, useState } from 'react';
import { cx } from 'utils';
import { storage } from 'utils/storage';
import { JSXChildren, JSXElement } from 'utils/jsx';
import { Icon } from './Icon';

type Props = {
  heading: JSXElement;
  className?: string;
  children: JSXChildren;
  open?: boolean;
  sessionKey?: string;
};

export const Spoiler = forwardRef<HTMLDetailsElement, Props>(function Spoiler(
  { heading, className, children, open: defaultOpen = false, sessionKey },
  innerRef,
) {
  const [isOpen, setIsOpen] = useState(() => {
    if (sessionKey) {
      return storage.getValue(sessionKey, defaultOpen);
    }

    return defaultOpen;
  });

  const onToggle = (e: React.SyntheticEvent<HTMLDetailsElement>) => {
    if (e.currentTarget.open === isOpen) {
      return;
    }

    setIsOpen(!isOpen);
    if (sessionKey) {
      storage.setValue(sessionKey, !isOpen);
    }
  };

  return (
    <details
      ref={innerRef}
      className={cx(className, 'w-full shadow-sm')}
      open={isOpen}
      onToggle={onToggle}
    >
      <summary className="cursor-pointer flex flex-row items-center gap-3 bg-neutral-100 dark:bg-neutral-900 px-2 py-2 rounded-sm">
        <Icon
          variant="chevron-up"
          className={cx('text-slate-500', isOpen ? 'rotate-180 transform' : '')}
        />

        {heading}
      </summary>
      <div className="bg-neutral-50 dark:bg-neutral-950 px-4 py-2 rounded-sm">{children}</div>
    </details>
  );
});
