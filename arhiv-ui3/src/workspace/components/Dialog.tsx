import { ComponentChildren } from 'preact';
import { createPortal } from 'preact/compat';
import { useEffect, useRef, useState } from 'preact/hooks';
import A11yDialog from 'a11y-dialog';
import { Callback, lockGlobalScroll } from '../../scripts/utils';

type DialogProps = {
  onHide: Callback;
  children: ComponentChildren;
};
export function Dialog({ onHide, children }: DialogProps) {
  const onHideRef = useRef(onHide);
  onHideRef.current = onHide;

  const [modalEl] = useState(() => {
    const rootEl = document.getElementById('modal-root');
    if (!rootEl) {
      throw new Error('modal root el not found');
    }

    const modalEl = document.createElement('div');
    modalEl.className = 'fixed z-50 inset-0 overflow-y-scroll backdrop-blur-md';
    rootEl.appendChild(modalEl);

    return modalEl;
  });

  useEffect(() => {
    const modal = new A11yDialog(modalEl);

    modal.show();

    modal.on('hide', onHideRef.current);

    return () => {
      modal.destroy();
      modalEl.remove();
    };
  }, []);

  useEffect(() => lockGlobalScroll(), []);

  return createPortal(<div className="modal">{children}</div>, modalEl);
}
