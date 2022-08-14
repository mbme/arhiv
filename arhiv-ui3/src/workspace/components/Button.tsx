import { ComponentChildren } from 'preact';
import { Callback } from '../../scripts/utils';

type ButtonProps = {
  variant: 'simple' | 'prime' | 'warn' | 'danger' | 'link'; // TODO rename link to text
  className?: string;
  onClick?: Callback;
  disabled?: boolean;
  children: ComponentChildren;
  title?: string;
};
export function Button({
  variant,
  className = '',
  onClick,
  disabled,
  children,
  title,
}: ButtonProps) {
  return (
    <button
      type="button"
      className={`btn btn-${variant} ${className}`}
      onClick={onClick}
      disabled={disabled}
      title={title}
    >
      {children}
    </button>
  );
}
