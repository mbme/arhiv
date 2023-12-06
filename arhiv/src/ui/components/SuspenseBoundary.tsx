import { Suspense } from 'react';
import { JSXChildren } from 'utils/jsx';
import { ErrorBoundary, ErrorRenderer } from 'components/ErrorBoundary';

type Props = {
  children: JSXChildren;
  fallback: React.ReactNode;
  renderError: ErrorRenderer;
};

export function SuspenseBoundary({ children, fallback, renderError }: Props) {
  return (
    <ErrorBoundary renderError={renderError}>
      <Suspense fallback={fallback}>{children}</Suspense>
    </ErrorBoundary>
  );
}
