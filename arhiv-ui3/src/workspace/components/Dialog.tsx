import { Dialog as HeadlessDialog } from '@headlessui/react';
import { Callback, cx } from '../../scripts/utils';
import { JSXChildren } from '../types';

type DialogProps = {
  onHide: Callback;
  alarming?: boolean;
  title: JSXChildren;
  children: JSXChildren;
};
export function Dialog({ onHide, alarming, title, children }: DialogProps) {
  return (
    <HeadlessDialog open onClose={onHide} className="modal-container">
      <div className="modal-overlay" />

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
    </HeadlessDialog>
  );
}
