import { DialogPanel, DialogTitle, Dialog as HeadlessDialog } from '@headlessui/react';
import { Callback, cx } from 'utils';
import { JSXChildren, JSXRef } from 'utils/jsx';
import { IconButton } from 'components/Button';

type DialogProps = {
  innerRef?: JSXRef<HTMLDivElement>;
  className?: string;
  contentClassName?: string;
  onHide: Callback;
  alarming?: boolean;
  title: JSXChildren;
  children: JSXChildren;
  buttons?: JSXChildren;
};
export function Dialog({
  innerRef,
  className,
  contentClassName,
  onHide,
  alarming,
  title,
  children,
  buttons,
}: DialogProps) {
  return (
    <HeadlessDialog ref={innerRef} open static onClose={onHide} className="modal-container">
      <div className="modal-overlay" />

      <div className="modal-dialog-container">
        <DialogPanel className={cx('modal-dialog', className)}>
          <DialogTitle
            className={cx('modal-title', {
              'is-alarming': alarming,
            })}
          >
            {title}

            <IconButton icon="x" size="sm" onClick={onHide} />
          </DialogTitle>

          <div className={cx('modal-content', contentClassName)}>{children}</div>

          {buttons && <div className="modal-buttons">{buttons}</div>}
        </DialogPanel>
      </div>
    </HeadlessDialog>
  );
}
