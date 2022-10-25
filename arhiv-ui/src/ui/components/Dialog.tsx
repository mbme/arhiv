import { createPortal } from 'preact/compat';
import { useEffect, useRef, useState } from 'preact/hooks';
import A11yDialog from 'a11y-dialog';
import { Callback, cx } from '../utils';
import { useId } from '../utils/hooks';
import { JSXChildren } from '../utils/jsx';

function lockGlobalScroll(): Callback {
  const documentEl = document.documentElement;
  const originalStyle = documentEl.style.cssText;
  const scrollTop = documentEl.scrollTop;

  // preserve scroll position and hide scroll
  documentEl.style.cssText = `position: fixed; left: 0; right: 0; overflow: hidden; top: -${scrollTop}px`;

  return () => {
    // restore scroll position
    documentEl.style.cssText = originalStyle;
    documentEl.scrollTop = scrollTop;
  };
}

type DialogProps = {
  onHide: Callback;
  alarming?: boolean;
  title: JSXChildren;
  children: JSXChildren;
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
