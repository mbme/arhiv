import { ComponentChildren } from 'preact';
import { createPortal } from 'preact/compat';
import { useEffect, useRef, useState } from 'preact/hooks';
import A11yDialog from 'a11y-dialog';
import { Callback, cx, lockGlobalScroll } from '../../scripts/utils';
import { useId } from '../hooks';

type DialogProps = {
  onHide: Callback;
  alarming?: boolean;
  title: ComponentChildren;
  children: ComponentChildren;
};
export function Dialog({ onHide, alarming, title, children }: DialogProps) {
  const [modalEl, setModalEl] = useState<HTMLElement | null>(null);

  const onHideRef = useRef(onHide);
  onHideRef.current = onHide;

  const rootEl = document.getElementById('modal-root');
  if (!rootEl) {
    throw new Error('modal root el not found');
  }

  useEffect(() => {
    if (!modalEl) {
      return;
    }

    const modal = new A11yDialog(modalEl);

    modal.show();

    modal.on('hide', onHideRef.current);

    return () => {
      modal.destroy();
    };
  }, [modalEl]);

  useEffect(() => {
    const onKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onHideRef.current();
      }
    };

    document.body.addEventListener('keydown', onKeydown);

    return () => {
      document.body.removeEventListener('keydown', onKeydown);
    };
  }, []);

  useEffect(() => lockGlobalScroll(), []);

  const id = useId();
  const titleId = `modal-title-${id}`;

  return createPortal(
    <div className="modal-container" ref={setModalEl} aria-labelledby={titleId} aria-hidden="true">
      <div data-a11y-dialog-hide className="modal-overlay"></div>

      <div role="document" className="modal-dialog">
        <h1
          id={titleId}
          className={cx('modal-title', {
            'is-alarming': alarming,
          })}
        >
          {title}
        </h1>

        {children}
      </div>
    </div>,
    rootEl
  );
}
