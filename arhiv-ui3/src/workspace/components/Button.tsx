import { ComponentChildren } from 'preact';
import { Callback, cx } from '../../scripts/utils';
import { Spinner } from './Spinner';

// TODO remove warn & danger;

type ButtonProps = {
  variant: 'simple' | 'prime' | 'warn' | 'danger' | 'text';
  className?: string;
  onClick?: Callback;
  disabled?: boolean;
  children: ComponentChildren;
  title?: string;
  loading?: boolean;
};
export function Button({
  variant,
  className = '',
  onClick,
  disabled,
  children,
  title,
  loading,
}: ButtonProps) {
  return (
    <button
      type="button"
      className={cx(className, {
        'btn btn-simple': variant === 'simple',
        'btn btn-prime': variant === 'prime',
        'btn btn-warn': variant === 'warn',
        'btn btn-danger': variant === 'danger',
        'btn btn-text': variant === 'text',
      })}
      onClick={onClick}
      disabled={disabled || loading}
      title={title}
    >
      {loading && <Spinner className="mr-2" />}

      {children}
    </button>
  );
}
