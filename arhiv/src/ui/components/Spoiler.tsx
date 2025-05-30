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
      className={cx(className, 'w-full shadow-xs')}
      open={isOpen}
      onToggle={onToggle}
    >
      <summary className="cursor-pointer flex flex-row items-center gap-3 var-bg-tertiary-color px-2 py-2 rounded-xs">
        <Icon
          variant="chevron-up"
          className={cx('text-slate-500', isOpen ? '' : 'rotate-180 transform')}
        />

        {heading}
      </summary>
      <div className="var-bg-tertiary-color px-4 py-2 rounded-xs">{children}</div>
    </details>
  );
});
