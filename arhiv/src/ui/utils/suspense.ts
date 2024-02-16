import {
  createContext,
  startTransition,
  useCallback,
  useContext,
  useDeferredValue,
  useEffect,
  useRef,
} from 'react';
import { APIRequest } from 'dto';
import { useForceRender } from 'utils/hooks';
import { API_ENDPOINT, doRPC, RPCResponse } from 'utils/rpc';

type Suspender<T> = { read: () => T };

export function suspensify<T>(promise: Promise<T>): Suspender<T> {
  type SuspenseStatus = { type: 'resolved'; value: T } | { type: 'rejected'; value: unknown };

  let status: SuspenseStatus | undefined;

  const pendingPromise = promise.then(
    (result) => {
      status = { type: 'resolved', value: result };
    },
    (err) => {
      status = { type: 'rejected', value: err };
    },
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

export const createSuspenseCache = () => new Map<string, Suspender<unknown>>();
type CardCache = ReturnType<typeof createSuspenseCache>;

export const SuspenseCacheContext = createContext<CardCache | undefined>(undefined);

export function useSuspenseQuery<Request extends APIRequest>(
  request: Request,
): {
  value: RPCResponse<Request>;
  isUpdating: boolean;
  triggerRefresh: (clearCache?: boolean) => void;
} {
  const cache = useContext(SuspenseCacheContext);
  if (!cache) {
    throw new Error('Suspense cache context is empty');
  }

  const queryName = JSON.stringify(request);

  useEffect(() => {
    return () => {
      cache.delete(queryName);
    };
  }, [cache, queryName]);

  let suspender = cache.get(queryName);
  if (!suspender) {
    suspender = suspensify(doRPC(API_ENDPOINT, request));
    cache.set(queryName, suspender);
  }
  const deferredSuspender = useDeferredValue(suspender);

  const value = deferredSuspender.read() as RPCResponse<Request>;

  const forceRender = useForceRender();
  const triggerRefresh = useCallback(
    (clearCache = false) => {
      if (clearCache) {
        cache.clear();
      } else {
        cache.delete(queryName);
      }

      startTransition(() => {
        forceRender();
      });
    },
    [cache, queryName, forceRender],
  );

  return {
    value,
    isUpdating: suspender !== deferredSuspender,
    triggerRefresh,
  };
}

export function useSuspenseImage(url: string): HTMLImageElement {
  const cache = useContext(SuspenseCacheContext);
  if (!cache) {
    throw new Error('Suspense cache context is empty');
  }

  // delete stale cache value if key changed
  const prevUrl = useRef(url);
  if (prevUrl.current !== url) {
    cache.delete(prevUrl.current);
    prevUrl.current = url;
  }

  let suspender = cache.get(url);
  if (!suspender) {
    const image = document.createElement('img');
    image.src = url;

    suspender = suspensify(
      image.decode().then(
        () => image,
        (err) => {
          console.warn('Failed to decode image', image, err);

          return image;
        },
      ),
    );
    cache.set(url, suspender);
  }
  const deferredSuspender = useDeferredValue(suspender);

  return deferredSuspender.read() as HTMLImageElement;
}
