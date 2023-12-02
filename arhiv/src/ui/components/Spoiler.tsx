import { useState } from 'react';
import { cx, getSessionValue, setSessionValue } from 'utils';
import { JSXChildren, JSXElement } from 'utils/jsx';
import { Icon } from './Icon';

type Props = {
  heading: JSXElement;
  className?: string;
  children: JSXChildren;
  open?: boolean;
  sessionKey?: string;
};

export function Spoiler({
  heading,
  className,
  children,
  open: defaultOpen = false,
  sessionKey,
}: Props) {
  const [isOpen, setIsOpen] = useState(() => {
    if (sessionKey) {
      return getSessionValue(sessionKey, defaultOpen);
    }

    return defaultOpen;
  });

  const onToggle = (e: React.SyntheticEvent<HTMLDetailsElement>) => {
    if (e.currentTarget.open === isOpen) {
      return;
    }

    setIsOpen(!isOpen);
    if (sessionKey) {
      setSessionValue(sessionKey, !isOpen);
    }
  };

  return (
    <details className={cx(className, 'w-full shadow-sm')} open={isOpen} onToggle={onToggle}>
      <summary className="cursor-pointer flex flex-row items-center gap-3 bg-neutral-100 px-2 py-2 rounded-sm">
        <Icon
          variant="chevron-up"
          className={cx('text-slate-500', isOpen ? '' : 'rotate-180 transform')}
        />

        {heading}
      </summary>
      <div className="bg-neutral-50 px-4 py-2 rounded-sm">{children}</div>
    </details>
  );
}
