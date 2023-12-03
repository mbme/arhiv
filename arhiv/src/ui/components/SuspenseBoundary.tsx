import { Suspense, useEffect } from 'react';
import { JSXChildren } from 'utils/jsx';
import { createSuspenseCache, SuspenseCacheContext } from 'utils/suspense';
import { ErrorBoundary, ErrorRenderer } from 'components/ErrorBoundary';

type CardCache = ReturnType<typeof createSuspenseCache>;
const CACHES = new Map<string, CardCache>();

type Props = {
  cacheId: string;
  children: JSXChildren;
  fallback: React.ReactNode;
  renderError: ErrorRenderer;
};

export function SuspenseBoundary({ cacheId, children, fallback, renderError }: Props) {
  const cache = CACHES.get(cacheId) || createSuspenseCache();
  CACHES.set(cacheId, cache);

  useEffect(() => {
    return () => {
      CACHES.delete(cacheId);
    };
  }, [cacheId]);

  return (
    <SuspenseCacheContext.Provider value={cache}>
      <ErrorBoundary renderError={renderError}>
        <Suspense fallback={fallback}>{children}</Suspense>
      </ErrorBoundary>
    </SuspenseCacheContext.Provider>
  );
}
