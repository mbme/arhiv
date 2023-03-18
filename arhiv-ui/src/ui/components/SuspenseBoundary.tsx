import { Suspense, useMemo } from 'react';
import { JSXChildren } from 'utils/jsx';
import { createSuspenseCache, SuspenseCacheContext } from 'utils/suspense';
import { ErrorBoundary } from 'components/ErrorBoundary';

type Props = {
  children: JSXChildren;
  fallback: React.ReactNode;
};

export function SuspenseBoundary({ children, fallback }: Props) {
  const cache = useMemo(() => createSuspenseCache(), []);

  return (
    <SuspenseCacheContext.Provider value={cache}>
      <ErrorBoundary>
        <Suspense fallback={fallback}>{children}</Suspense>
      </ErrorBoundary>
    </SuspenseCacheContext.Provider>
  );
}
