import * as RadixDialog from '@radix-ui/react-dialog';
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
    <RadixDialog.Root
      open
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          onHide();
        }
      }}
    >
      <RadixDialog.Portal>
        <RadixDialog.Overlay className="modal-backdrop" />

        <RadixDialog.Content className="modal-container">
          <div className="modal-dialog">
            <RadixDialog.Title
              className={cx('modal-title', {
                'is-alarming': alarming,
              })}
            >
              {title}
            </RadixDialog.Title>

            {children}
          </div>
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  );
}
