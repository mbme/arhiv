import { Suspense, useMemo } from 'react';
import { JSXChildren } from 'utils/jsx';
import { Suspender, SuspenseCacheContext } from 'utils/suspense';
import { Icon } from 'components/Icon';

type Props = {
  children: JSXChildren;
};

export function SuspenseBoundary({ children }: Props) {
  const cache = useMemo(() => new Map<string, Suspender<unknown>>(), []);

  // TODO error boundary

  return (
    <SuspenseCacheContext.Provider value={cache}>
      <Suspense fallback={<Icon variant="spinner" className="mb-8" />}>{children}</Suspense>
    </SuspenseCacheContext.Provider>
  );
}
