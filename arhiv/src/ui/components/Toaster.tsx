import { useEffect, useReducer, useRef } from 'react';
import { createPortal } from 'react-dom';
import { cx } from 'utils';
import { IconButton } from 'components/Button';

const DEFAULT_TOAST_TIMEOUT_MS = 5000;

type ToastOptions = {
  level: 'info' | 'warn';
  message: string;
  timeoutMs?: number;
};

type Toast = ToastOptions & { id: number; createdAtMs: number; timeoutMs: number };
let toastId = 0;

class ToastEvent extends CustomEvent<ToastOptions> {
  constructor(public readonly options: ToastOptions) {
    super('toast', { detail: options });
  }
}

export function showToast(options: ToastOptions) {
  document.dispatchEvent(new ToastEvent(options));
}

type ToasterAction =
  | {
      type: 'open';
      toast: Toast;
    }
  | {
      type: 'close';
      id: number;
    };

function toasterReducer(state: Toast[], action: ToasterAction): Toast[] {
  switch (action.type) {
    case 'open': {
      return [...state, action.toast];
    }
    case 'close': {
      return state.filter((toast) => toast.id !== action.id);
    }
    default: {
      throw new Error(`unknown action ${JSON.stringify(action)}`);
    }
  }
}

export function Toaster() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [toasts, dispatch] = useReducer(toasterReducer, []);

  useEffect(() => {
    const onToast = (e: Event) => {
      const toastOptions = (e as ToastEvent).detail;
      const toast = {
        ...toastOptions,
        id: (toastId += 1),
        createdAtMs: Date.now(),
        timeoutMs: toastOptions.timeoutMs ?? DEFAULT_TOAST_TIMEOUT_MS,
      };

      dispatch({ type: 'open', toast });
    };

    document.addEventListener('toast', onToast);

    return () => {
      document.removeEventListener('toast', onToast);
    };
  }, []);

  useEffect(() => {
    if (!toasts.length) {
      return;
    }

    const intervalId = setInterval(() => {
      if (containerRef.current?.matches(':hover')) {
        return;
      }

      for (const toast of toasts) {
        if (Date.now() - toast.createdAtMs > toast.timeoutMs) {
          dispatch({ type: 'close', id: toast.id });
        }
      }
    }, 200);

    return () => {
      clearInterval(intervalId);
    };
  }, [toasts]);

  return createPortal(
    <div
      className="fixed top-10 left-1/2 transform -translate-x-1/2 flex flex-col gap-3 pl-4 pb-4 z-100"
      ref={containerRef}
    >
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={cx(
            'px-4 py-2 text-sm rounded-sm shadow-sm hover:shadow-lg cursor-default min-w-[15rem] group flex flex-row',
            {
              'bg-emerald-500': toast.level === 'info',
              'bg-orange-600': toast.level === 'warn',
            },
          )}
        >
          <div className="grow">{toast.message}</div>

          <IconButton
            icon="x"
            size="sm"
            className="invisible group-hover:visible"
            onClick={() => {
              dispatch({ type: 'close', id: toast.id });
            }}
          />
        </div>
      ))}
    </div>,
    document.body,
  );
}
