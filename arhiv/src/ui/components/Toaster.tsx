import { useEffect, useReducer, useRef } from 'react';
import { cx } from 'utils';
import { IconButton } from 'components/Button';

const TOAST_TIMEOUT_MS = 5000;

type ToastOptions = {
  level: 'info' | 'warn';
  message: string;
};

type Toast = ToastOptions & { id: number; createdAtMs: number };
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
      const toast = {
        ...(e as ToastEvent).detail,
        id: (toastId += 1),
        createdAtMs: Date.now(),
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
        if (Date.now() - toast.createdAtMs > TOAST_TIMEOUT_MS) {
          dispatch({ type: 'close', id: toast.id });
        }
      }
    }, 200);

    return () => {
      clearInterval(intervalId);
    };
  }, [toasts]);

  return (
    <div className="fixed bottom-0 left-0 flex flex-col gap-3 pl-4 pb-4" ref={containerRef}>
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={cx(
            'px-4 py-2 text-sm rounded shadow hover:shadow-lg cursor-default min-w-[15rem] group flex flex-row',
            {
              'bg-sky-50 text-sky-700': toast.level === 'info',
              'bg-orange-200 text-amber-700': toast.level === 'warn',
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
    </div>
  );
}
