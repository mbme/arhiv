import { Dialog as HeadlessDialog } from '@headlessui/react';
import { Callback, cx } from 'utils';
import { JSXChildren, JSXRef } from 'utils/jsx';

type DialogProps = {
  innerRef?: JSXRef<HTMLDivElement>;
  onHide: Callback;
  alarming?: boolean;
  title: JSXChildren;
  children: JSXChildren;
  buttons?: JSXChildren;
};
export function Dialog({ innerRef, onHide, alarming, title, children, buttons }: DialogProps) {
  return (
    <HeadlessDialog ref={innerRef} open static onClose={onHide} className="modal-container">
      <div className="modal-overlay" />

      <div className="modal-dialog-container">
        <HeadlessDialog.Panel className="modal-dialog">
          <HeadlessDialog.Title
            className={cx('modal-title', {
              'is-alarming': alarming,
            })}
          >
            {title}
          </HeadlessDialog.Title>

          <div className="modal-content">{children}</div>

          {buttons && <div className="modal-buttons">{buttons}</div>}
        </HeadlessDialog.Panel>
      </div>
    </HeadlessDialog>
  );
}
