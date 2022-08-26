import { EffectCallback, useEffect, useRef, useState } from 'preact/hooks';
import { Callback } from '../scripts/utils';

type Inputs = ReadonlyArray<unknown>;

type Options = {
  refreshIfChange?: Inputs;
  refreshOnMount?: boolean;
};

type QueryHookResult<TResult> = {
  result?: TResult;
  inProgress: boolean;
  error?: unknown;
  triggerRefresh: Callback;
};

export function useQuery<TResult>(
  cb: (signal: AbortSignal) => Promise<TResult>,
  options?: Options
): QueryHookResult<TResult> {
  const cbRef = useRef(cb);
  cbRef.current = cb;

  const [inProgress, setInProgress] = useState(false);
  const [result, setResult] = useState<TResult>();
  const [error, setError] = useState<unknown>();

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
        setError(error);
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
