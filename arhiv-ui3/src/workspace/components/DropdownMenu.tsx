import { useState } from 'react';
import {
  FloatingFocusManager,
  offset,
  shift,
  useClick,
  useDismiss,
  useFloating,
  useInteractions,
  useRole,
} from '@floating-ui/react-dom-interactions';
import { Callback, cx } from '../../scripts/utils';
import { Icon, IconVariant } from './Icon';
import { IconButton } from './Button';

type MenuItem = {
  text: string;
  icon?: IconVariant;
  onClick: Callback;
};

type DropdownMenuProps = {
  options: readonly MenuItem[];
};

export function DropdownMenu({ options }: DropdownMenuProps) {
  const [open, setOpen] = useState(false);

  const { x, y, reference, floating, strategy, context } = useFloating({
    placement: 'bottom-start',
    middleware: [offset(10), shift()],
    open,
    onOpenChange: setOpen,
  });

  const { getReferenceProps, getFloatingProps, getItemProps } = useInteractions([
    useRole(context, {
      role: 'menu',
    }),
    useClick(context),
    useDismiss(context),
  ]);

  return (
    <>
      <IconButton icon="more" size="lg" ref={reference} {...getReferenceProps()} />

      {open && (
        <FloatingFocusManager context={context}>
          <div
            ref={floating}
            className={cx(
              'bg-white rounded min-w-[10rem] flex flex-col gap-2 drop-shadow py-2',
              {}
            )}
            style={{
              position: strategy,
              top: y ?? 0,
              left: x ?? 0,
            }}
            {...getFloatingProps()}
          >
            {options.map((option, index) => (
              <button
                type="button"
                key={index}
                className="flex items-center gap-2 cursor-pointer text-blue-700 hover:bg-sky-100 px-4 py-2 whitespace-nowrap"
                role="menuitem"
                {...getItemProps({
                  onClick: () => {
                    option.onClick();
                    setOpen(false);
                  },
                })}
              >
                {option.icon ? <Icon variant={option.icon} /> : <div className="w-5" />}

                {option.text}
              </button>
            ))}
          </div>
        </FloatingFocusManager>
      )}
    </>
  );
}
