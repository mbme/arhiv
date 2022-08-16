import { ComponentChildren } from 'preact';
import { Callback } from '../../scripts/utils';
import { Spinner } from './Spinner';

type ButtonProps = {
  variant: 'simple' | 'prime' | 'warn' | 'danger' | 'link'; // TODO rename link to text
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
      className={`btn btn-${variant} ${className}`}
      onClick={onClick}
      disabled={disabled || loading}
      title={title}
    >
      {loading && <Spinner className="mr-2" />}

      {children}
    </button>
  );
}
