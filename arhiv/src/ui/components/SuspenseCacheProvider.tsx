import { useEffect } from 'react';
import { JSXChildren } from 'utils/jsx';
import { createSuspenseCache, SuspenseCacheContext } from 'utils/suspense';

type CardCache = ReturnType<typeof createSuspenseCache>;
const CACHES = new Map<string, CardCache>();

type Props = {
  cacheId: string;
  children: JSXChildren;
};

export function SuspenseCacheProvider({ cacheId, children }: Props) {
  const cache = CACHES.get(cacheId) || createSuspenseCache();
  CACHES.set(cacheId, cache);

  useEffect(() => {
    return () => {
      CACHES.delete(cacheId);
    };
  }, [cacheId]);

  return <SuspenseCacheContext.Provider value={cache}>{children}</SuspenseCacheContext.Provider>;
}
