import { createContext, useCallback, useContext, useDeferredValue } from 'react';
import { Callback } from 'utils';
import { useForceRender } from 'utils/hooks';

export type Suspender<T> = { read: () => T };

export function suspensify<T>(promise: Promise<T>): Suspender<T> {
  type SuspenseStatus = { type: 'resolved'; value: T } | { type: 'rejected'; value: unknown };

  let status: SuspenseStatus | undefined;

  const pendingPromise = promise.then(
    (result) => {
      status = { type: 'resolved', value: result };
    },
    (err) => {
      status = { type: 'rejected', value: err };
    }
  );

  return {
    read() {
      if (!status) {
        throw pendingPromise;
      }

      if (status.type === 'rejected') {
        throw status.value;
      }

      return status.value;
    },
  };
}

export const SuspenseCacheContext = createContext(new Map<string, Suspender<unknown>>());

export function useSuspense<T>(
  cacheKey: string,
  factory: () => Promise<T>
): { value: T; isUpdating: boolean; triggerRefresh: Callback } {
  const cache = useContext(SuspenseCacheContext);

  // TODO remove previous value if cacheKey changes

  let suspender = cache.get(cacheKey);
  if (!suspender) {
    suspender = suspensify(factory());
    cache.set(cacheKey, suspender);
  }
  const deferredSuspender = useDeferredValue(suspender);

  const value = deferredSuspender.read() as T;

  const forceRender = useForceRender();
  const triggerRefresh = useCallback(() => {
    console.error('trigger refresh');
    cache.delete(cacheKey);
    forceRender();
  }, [cacheKey, forceRender]);

  return {
    value,
    isUpdating: suspender !== deferredSuspender,
    triggerRefresh,
  };
}
