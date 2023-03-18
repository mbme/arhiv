import { Suspense, useMemo } from 'react';
import { JSXChildren } from 'utils/jsx';
import { Suspender, SuspenseCacheContext } from 'utils/suspense';
import { ErrorBoundary } from 'components/ErrorBoundary';

type Props = {
  children: JSXChildren;
  fallback: React.ReactNode;
};

export function SuspenseBoundary({ children, fallback }: Props) {
  const cache = useMemo(() => new Map<string, Suspender<unknown>>(), []);

  return (
    <SuspenseCacheContext.Provider value={cache}>
      <ErrorBoundary>
        <Suspense fallback={fallback}>{children}</Suspense>
      </ErrorBoundary>
    </SuspenseCacheContext.Provider>
  );
}
