import { useEffect, useRef, useState } from 'preact/hooks';

export function useQuery<TResult>(
  cb: (signal: AbortSignal) => Promise<TResult>,
  inputs: readonly unknown[]
): { result?: TResult; inProgress: boolean; error?: unknown } {
  const cbRef = useRef(cb);
  cbRef.current = cb;

  const [inProgress, setInProgress] = useState(false);
  const [result, setResult] = useState<TResult>();
  const [error, setError] = useState<unknown>();

  useEffect(() => {
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
  }, inputs); // eslint-disable-line react-hooks/exhaustive-deps

  return {
    result,
    error,
    inProgress,
  };
}
