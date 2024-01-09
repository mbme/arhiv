import { Component } from 'react';
import { JSXChildren, JSXElement } from 'utils/jsx';

export type ErrorRenderer = (error: unknown) => JSXElement;

type Props = {
  children: JSXChildren;
  renderError: ErrorRenderer;
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
    const { children, renderError } = this.props;
    const { error } = this.state;

    return <>{error ? renderError(error) : children}</>;
  }
}
