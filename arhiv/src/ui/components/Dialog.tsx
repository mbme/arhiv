import { Dialog as HeadlessDialog } from '@headlessui/react';
import { Callback, cx } from 'utils';
import { JSXChildren, JSXRef } from 'utils/jsx';
import { IconButton } from 'components/Button';

type DialogProps = {
  innerRef?: JSXRef<HTMLDivElement>;
  className?: string;
  onHide: Callback;
  alarming?: boolean;
  title: JSXChildren;
  children: JSXChildren;
  buttons?: JSXChildren;
};
export function Dialog({
  innerRef,
  className,
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
        <HeadlessDialog.Panel className={cx('modal-dialog', className)}>
          <HeadlessDialog.Title
            className={cx('modal-title', {
              'is-alarming': alarming,
            })}
          >
            {title}

            <IconButton icon="x" size="sm" onClick={onHide} />
          </HeadlessDialog.Title>

          <div className="modal-content">{children}</div>

          {buttons && <div className="modal-buttons">{buttons}</div>}
        </HeadlessDialog.Panel>
      </div>
    </HeadlessDialog>
  );
}
