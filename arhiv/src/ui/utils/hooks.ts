import {
  MutableRefObject,
  EffectCallback,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { Signal, effect } from '@preact/signals-core';
import { shallowEqual } from 'shallow-equal';
import { Callback, debounce, JSONValue, throttle } from './index';
import { storage } from './storage';

export type Inputs = ReadonlyArray<unknown>;

export function useShallowMemo<T>(value: T): T {
  const valueRef = useRef(value);

  if (value === valueRef.current) {
    return value;
  }

  if (
    value instanceof Object &&
    valueRef.current instanceof Object &&
    shallowEqual(value, valueRef.current)
  ) {
    return valueRef.current;
  }

  valueRef.current = value;

  return valueRef.current;
}

type Options<TResult> = {
  refreshIfChange?: Inputs;
  refreshOnMount?: boolean;
  onSuccess?: (result: TResult) => void;
};

type QueryHookResult<TResult> = {
  result?: TResult;
  inProgress: boolean;
  error?: Error | string;
  triggerRefresh: Callback;
  requestTs: number; // millis
};

// TODO refetch on focus, refetch on reconnect, cache
// inspired by https://swr.vercel.app/docs/api
export function useQuery<TResult>(
  cb: (signal: AbortSignal) => Promise<TResult>,
  options: Options<TResult> = {},
): QueryHookResult<TResult> {
  const cbRef = useLatestRef(cb);
  const optionsRef = useLatestRef(options);

  const [inProgress, setInProgress] = useState(false);
  const [result, setResult] = useState<TResult>();
  const [error, setError] = useState<Error | string>();
  const [requestTs, setRequestTs] = useState(0);

  const [counter, setCounter] = useState(0);

  useEffect(() => {
    const refreshOnMount = optionsRef.current.refreshOnMount ?? true;
    if (counter === 0 && !refreshOnMount) {
      return;
    }

    const controller = new AbortController();
    setInProgress(true);
    setRequestTs(Date.now());

    cbRef.current(controller.signal).then(
      (result) => {
        setResult(result);
        setError(undefined);
        setInProgress(false);
        optionsRef.current.onSuccess?.(result);
      },
      (error) => {
        setResult(undefined);
        setError(error as Error | string);
        setInProgress(false);
      },
    );

    return () => {
      controller.abort();
      setInProgress(false);
    };
  }, [counter, cbRef, optionsRef]);

  const inputsMemo = useShallowMemo(options.refreshIfChange ?? []);
  useUpdateEffect(() => {
    setCounter(counter + 1);
  }, [inputsMemo]);

  const triggerRefresh = useCallback(() => {
    setCounter((counter) => counter + 1);
  }, []);

  return {
    result,
    error,
    inProgress,
    requestTs,
    triggerRefresh,
  };
}

export function useLatestRef<T>(value: T): MutableRefObject<T> {
  const valueRef = useRef(value);
  valueRef.current = value;

  return valueRef;
}

export function useAbortSignal() {
  const [abortController] = useState(() => new AbortController());

  useEffect(() => {
    return () => {
      abortController.abort();
    };
  }, [abortController]);

  return abortController.signal;
}

export function useUpdateEffect(effect: EffectCallback, deps: Inputs) {
  const isFirstRef = useRef(true);
  useEffect(() => {
    if (isFirstRef.current) {
      isFirstRef.current = false;
      return;
    }

    return effect();
  }, deps); // eslint-disable-line react-hooks/exhaustive-deps
}

export function useTimeout(cb: Callback, timeoutMs: number, enabled: boolean): void {
  const cbRef = useRef(cb);
  cbRef.current = cb;

  useEffect(() => {
    if (!enabled) {
      return;
    }

    const timeoutId = setTimeout(() => {
      cbRef.current();
    }, timeoutMs);

    return () => {
      clearTimeout(timeoutId);
    };
  }, [timeoutMs, enabled]);
}

// TODO get rid of any
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function useDebouncedCallback<Args extends any[]>(
  callback: (...args: Args) => void,
  waitFor: number,
) {
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  const debouncedCallback = useMemo(() => {
    if (waitFor < 1) {
      return (...args: Args) => {
        callbackRef.current(...args);
      };
    }

    return debounce<Args, (...args: Args) => void>((...args) => {
      callbackRef.current(...args);
    }, waitFor);
  }, [waitFor]);

  return debouncedCallback;
}

export function useSessionState<T extends JSONValue>(
  key: string,
  initialValue: T,
): [T, (newValue: T) => void] {
  const initialKeyRef = useRef(key);
  if (initialKeyRef.current !== key) {
    throw new Error(`key changed from "${initialKeyRef.current}" to "${key}"`);
  }

  const [value, setValue] = useState<T>(() => storage.getValue(key, initialValue));

  const updateValue = useCallback(
    (newValue: T) => {
      if (newValue === null) {
        storage.removeValue(key);
      } else {
        storage.setValue(key, newValue);
      }
      setValue(newValue);
    },
    [key],
  );

  return [value, updateValue];
}

export function useToggle(initialValue: boolean): [boolean, Callback] {
  const [value, setValue] = useState(initialValue);

  const toggleValue = useCallback(() => {
    setValue((currentValue) => !currentValue);
  }, []);

  return [value, toggleValue];
}

export function useUnsavedChangesWarning(warn: boolean) {
  useEffect(() => {
    if (!warn) {
      return;
    }

    const onBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();

      return (event.returnValue = 'Page has unsaved changes. Are you sure you want to exit?');
    };

    window.addEventListener('beforeunload', onBeforeUnload, { capture: true });

    return () => {
      window.removeEventListener('beforeunload', onBeforeUnload, { capture: true });
    };
  }, [warn]);
}

export function useScrollRestoration(el: HTMLElement | null, key: string) {
  useEffect(() => {
    if (!el) {
      return;
    }

    const [scrollTop, scrollLeft] = storage.getValue(key, [0, 0]);

    const originalScrollBehavior = el.style.scrollBehavior;

    el.style.scrollBehavior = 'auto';
    el.scrollTo({
      top: scrollTop,
      left: scrollLeft,
    });

    el.style.scrollBehavior = originalScrollBehavior;

    const onScroll = throttle(() => {
      storage.setValue(key, [el.scrollTop, el.scrollLeft]);
    }, 350);

    // TODO also listen for resize?

    el.addEventListener('scroll', onScroll, { passive: true });

    return () => {
      el.removeEventListener('scroll', onScroll);
      storage.removeValue(key);
    };
  }, [el, key]);
}

export function useForceRender(): Callback {
  const [, setCounter] = useState(0);

  return useCallback(() => {
    setCounter((value) => value + 1);
  }, []);
}

export function useIsPageVisible(): boolean {
  const [isPageVisible, setIsPageVisible] = useState(() => document.visibilityState === 'visible');

  useEffect(() => {
    const visibilityChangeHandler = () => {
      setIsPageVisible(document.visibilityState === 'visible');
    };

    document.addEventListener('visibilitychange', visibilityChangeHandler);

    return () => {
      document.removeEventListener('visibilitychange', visibilityChangeHandler);
    };
  }, []);

  return isPageVisible;
}

export function usePageVisibilityTracker(onPageVisibilityChange: (visible: boolean) => void) {
  const onPageVisibilityChangeRef = useLatestRef(onPageVisibilityChange);

  useEffect(() => {
    const visibilityChangeHandler = () => {
      onPageVisibilityChangeRef.current(document.visibilityState === 'visible');
    };

    document.addEventListener('visibilitychange', visibilityChangeHandler);

    return () => {
      document.removeEventListener('visibilitychange', visibilityChangeHandler);
    };
  }, [onPageVisibilityChangeRef]);
}

export function useKeydown(
  el: HTMLElement | undefined | null,
  onKeyDown: (e: KeyboardEvent) => void,
) {
  const onKeyDownRef = useLatestRef(onKeyDown);

  useEffect(() => {
    if (!el) {
      return;
    }

    const onKeyDown = (e: KeyboardEvent) => {
      onKeyDownRef.current(e);
    };

    el.addEventListener('keydown', onKeyDown);

    return () => {
      el.removeEventListener('keydown', onKeyDown);
    };
  }, [el, onKeyDownRef]);
}

export function useSignal<T>(signal: Signal<T>): T {
  const [value, setValue] = useState(signal.value);

  useEffect(() => {
    setValue(signal.value);

    const unsub = effect(() => {
      setValue(signal.value);
    });

    return () => {
      unsub();
    };
  }, [signal]);

  return value;
}

export function useClipboardPasteHandler(handler: (data: DataTransfer) => Promise<void> | void) {
  const handlerRef = useLatestRef(handler);

  useEffect(() => {
    const onPaste = (event: ClipboardEvent) => {
      if (event.clipboardData?.items) {
        void handlerRef.current(event.clipboardData);
      }
    };

    document.addEventListener('paste', onPaste);

    return () => {
      document.removeEventListener('paste', onPaste);
    };
  }, [handlerRef]);
}
