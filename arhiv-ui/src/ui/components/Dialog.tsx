import { Dialog as HeadlessDialog } from '@headlessui/react';
import { Callback, cx } from 'utils';
import { JSXChildren, JSXRef } from 'utils/jsx';

type DialogProps = {
  innerRef?: JSXRef<HTMLDivElement>;
  onHide: Callback;
  alarming?: boolean;
  title: JSXChildren;
  children: JSXChildren;
};
export function Dialog({ innerRef, onHide, alarming, title, children }: DialogProps) {
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

          {children}
        </HeadlessDialog.Panel>
      </div>
    </HeadlessDialog>
  );
}
