import { Component } from 'react';
import { JSXChildren } from 'utils/jsx';
import { QueryError } from 'components/QueryError';

type Props = {
  children: JSXChildren;
};

type State = {
  error?: unknown;
};

export class ErrorBoundary extends Component<Props, State> {
  override state: State = {
    error: undefined,
  };

  static getDerivedStateFromError(error: unknown) {
    return { error };
  }

  override render() {
    const { children } = this.props;
    const { error } = this.state;

    return (
      <>
        {error && <QueryError error={error} />}
        {children}
      </>
    );
  }
}
