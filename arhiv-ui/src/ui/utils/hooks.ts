import {
  EffectCallback,
  StateUpdater,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'preact/hooks';
import {
  ensure,
  Callback,
  debounce,
  getSessionValue,
  JSONValue,
  newId,
  setSessionValue,
} from './index';

type Inputs = ReadonlyArray<unknown>;

type Options = {
  refreshIfChange?: Inputs;
  refreshOnMount?: boolean;
};

type QueryHookResult<TResult> = {
  result?: TResult;
  inProgress: boolean;
  error?: Error | string;
  triggerRefresh: Callback;
};

// TODO refetch on focus, refetch on reconnect, cache
export function useQuery<TResult>(
  cb: (signal: AbortSignal) => Promise<TResult>,
  options?: Options
): QueryHookResult<TResult> {
  const cbRef = useRef(cb);
  cbRef.current = cb;

  const [inProgress, setInProgress] = useState(false);
  const [result, setResult] = useState<TResult>();
  const [error, setError] = useState<Error | string>();

  const [counter, setCounter] = useState(0);

  useEffect(() => {
    const refreshOnMount = options?.refreshOnMount ?? true;
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
  }, [counter]); // eslint-disable-line react-hooks/exhaustive-deps

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

// https://usehooks-ts.com/react-hook/use-is-first-render
function useIsFirstRender(): boolean {
  const isFirstRef = useRef(true);

  if (isFirstRef.current) {
    isFirstRef.current = false;

    return true;
  }

  return false;
}

// https://usehooks-ts.com/react-hook/use-update-effect
export function useUpdateEffect(effect: EffectCallback, deps: Inputs) {
  const isFirst = useIsFirstRender();

  useEffect(() => {
    if (!isFirst) {
      return effect();
    }
  }, deps); // eslint-disable-line react-hooks/exhaustive-deps
}

export function useId(): number {
  const [id] = useState(() => newId());

  return id;
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
  ensure(waitFor > 1, 'waitFor must be positive number');

  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  const debouncedCallback = useMemo(
    () =>
      debounce<Args, (...args: Args) => void>((...args) => {
        callbackRef.current(...args);
      }, waitFor),
    [waitFor]
  );

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

export function useUnsavedChangesWarning() {
  useEffect(() => {
    const onBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();

      return (event.returnValue = 'Page has unsaved changes. Are you sure you want to exit?');
    };

    window.addEventListener('beforeunload', onBeforeUnload, { capture: true });

    return () => {
      window.removeEventListener('beforeunload', onBeforeUnload, { capture: true });
    };
  }, []);
}
