import { Suspense, useMemo } from 'react';
import { JSXChildren } from 'utils/jsx';
import { Suspender, SuspenseCacheContext } from 'utils/suspense';
import { Icon } from 'components/Icon';
import { ErrorBoundary } from 'components/ErrorBoundary';

type Props = {
  children: JSXChildren;
};

export function SuspenseBoundary({ children }: Props) {
  const cache = useMemo(() => new Map<string, Suspender<unknown>>(), []);

  return (
    <SuspenseCacheContext.Provider value={cache}>
      <ErrorBoundary>
        <Suspense fallback={<Icon variant="spinner" className="mb-8" />}>{children}</Suspense>
      </ErrorBoundary>
    </SuspenseCacheContext.Provider>
  );
}
