import {
  MutableRefObject,
  EffectCallback,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import {
  Callback,
  debounce,
  getSessionValue,
  JSONValue,
  removeSessionValue,
  setSessionValue,
  throttle,
} from './index';
import { StateUpdater } from './jsx';

type Inputs = ReadonlyArray<unknown>;

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
};

// TODO refetch on focus, refetch on reconnect, cache
// inspired by https://swr.vercel.app/docs/api
export function useQuery<TResult>(
  cb: (signal: AbortSignal) => Promise<TResult>,
  options: Options<TResult> = {}
): QueryHookResult<TResult> {
  const cbRef = useLatestRef(cb);
  const optionsRef = useLatestRef(options);

  const [inProgress, setInProgress] = useState(false);
  const [result, setResult] = useState<TResult>();
  const [error, setError] = useState<Error | string>();

  const [counter, setCounter] = useState(0);

  useEffect(() => {
    const refreshOnMount = optionsRef.current.refreshOnMount ?? true;
    if (counter === 0 && !refreshOnMount) {
      return;
    }

    const controller = new AbortController();
    setInProgress(true);

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
      }
    );

    return () => {
      controller.abort();
      setInProgress(false);
    };
  }, [counter, cbRef, optionsRef]);

  useUpdateEffect(() => {
    setCounter(counter + 1);
  }, options?.refreshIfChange ?? []);

  return {
    result,
    error,
    inProgress,
    triggerRefresh() {
      setCounter(counter + 1);
    },
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

    const timeoutId = setTimeout(() => cbRef.current(), timeoutMs);

    return () => {
      clearTimeout(timeoutId);
    };
  }, [timeoutMs, enabled]);
}

export function useDebouncedCallback<Args extends any[]>(
  callback: (...args: Args) => void,
  waitFor: number
) {
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  const debouncedCallback = useMemo(() => {
    if (waitFor < 1) {
      return (...args: Args) => callbackRef.current(...args);
    }

    return debounce<Args, (...args: Args) => void>((...args) => {
      callbackRef.current(...args);
    }, waitFor);
  }, [waitFor]);

  return debouncedCallback;
}

export function useSessionState<T extends JSONValue>(
  key: string,
  initialValue: T
): [T, StateUpdater<T>] {
  const initialKeyRef = useRef(key);
  if (initialKeyRef.current !== key) {
    throw new Error(`key changed from "${initialKeyRef.current}" to "${key}"`);
  }

  const [value, setValue] = useState<T>(() => getSessionValue(key, initialValue));

  useEffect(() => {
    setSessionValue(key, value);
  }, [key, value]);

  return [value, setValue];
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
  useLayoutEffect(() => {
    if (!el) {
      return;
    }

    const [scrollTop, scrollLeft] = getSessionValue(key, [0, 0]);

    const originalScrollBehavior = el.style.scrollBehavior;

    el.style.scrollBehavior = 'auto';
    el.scrollTo({
      top: scrollTop,
      left: scrollLeft,
    });

    el.style.scrollBehavior = originalScrollBehavior;

    const onScroll = throttle(() => {
      setSessionValue(key, [el.scrollTop, el.scrollLeft]);
    }, 350);

    // TODO also listen for resize?

    el.addEventListener('scroll', onScroll, { passive: true });

    return () => {
      el.removeEventListener('scroll', onScroll);
      removeSessionValue(key);
    };
  }, [el, key]);
}

export function useForceRender(): Callback {
  const [, setCounter] = useState(0);

  return useCallback(() => setCounter((value) => value + 1), []);
}

function isShallowEqualArray(arr1: unknown[], arr2: unknown[]): boolean {
  if (arr1.length !== arr2.length) {
    return false;
  }

  for (let i = 0; i < arr1.length; i += 1) {
    if (arr1[i] !== arr2[i]) {
      return false;
    }
  }

  return true;
}

export function useImmediateEffect(cb: Callback, deps: unknown[] = []): void {
  const prevValueRef = useRef(deps);

  if (!isShallowEqualArray(prevValueRef.current, deps)) {
    prevValueRef.current = deps;
    cb();
  }
}
