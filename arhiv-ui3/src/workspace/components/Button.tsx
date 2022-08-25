import { ComponentChildren } from 'preact';
import { Callback, cx } from '../../scripts/utils';
import { Spinner } from './Spinner';

// TODO remove warn & danger;

type ButtonProps = {
  variant: 'simple' | 'prime' | 'text';
  color?: 'warn' | 'danger';
  className?: string;
  onClick?: Callback;
  disabled?: boolean;
  children: ComponentChildren;
  title?: string;
  loading?: boolean;
};
export function Button({
  variant,
  color,
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
        'btn btn-text': variant === 'text',
        'btn-warn': color === 'warn',
        'btn-danger': color === 'danger',
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
