import { cx } from 'utils';
import { JSXChildren, JSXElement } from 'utils/jsx';
import { useToggle } from 'utils/hooks';
import { Icon } from './Icon';

type Props = {
  heading: JSXElement;
  className?: string;
  children: JSXChildren;
  open?: boolean;
};

export function Spoiler({ heading, className, children, open: defaultOpen = false }: Props) {
  const [isOpen, toggleOpen] = useToggle(defaultOpen);

  return (
    <details className={cx(className, 'w-full shadow-sm')} open={isOpen} onToggle={toggleOpen}>
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
