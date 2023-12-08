import { useEffect } from 'react';
import { useShallowMemo } from 'utils/hooks';
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

  return <SuspenseCacheContext.Provider value={cache}>{children}</SuspenseCacheContext.Provider>;
}

export function useSuspenseCacheCleaner(cacheIds: string[]) {
  const ids = useShallowMemo(cacheIds);

  useEffect(() => {
    for (const key of CACHES.keys()) {
      if (!ids.includes(key)) {
        CACHES.delete(key);
      }
    }
  }, [ids]);
}
