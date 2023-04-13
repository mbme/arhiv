import { Suspense, useMemo } from 'react';
import { JSXChildren } from 'utils/jsx';
import { createSuspenseCache, SuspenseCacheContext } from 'utils/suspense';
import { ErrorBoundary, ErrorRenderer } from 'components/ErrorBoundary';

type Props = {
  children: JSXChildren;
  fallback: React.ReactNode;
  renderError: ErrorRenderer;
};

export function SuspenseBoundary({ children, fallback, renderError }: Props) {
  const cache = useMemo(() => createSuspenseCache(), []);

  return (
    <SuspenseCacheContext.Provider value={cache}>
      <ErrorBoundary renderError={renderError}>
        <Suspense fallback={fallback}>{children}</Suspense>
      </ErrorBoundary>
    </SuspenseCacheContext.Provider>
  );
}
