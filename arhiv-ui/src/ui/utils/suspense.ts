import { createContext, useContext } from 'react';

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

export function useSuspense<T>(cacheKey: string, factory: () => Promise<T>): T {
  const cache = useContext(SuspenseCacheContext);

  const cachedSuspender = cache.get(cacheKey);
  if (cachedSuspender) {
    return cachedSuspender.read() as T;
  }

  const suspender = suspensify(factory());
  cache.set(cacheKey, suspender);

  return suspender.read();
}
