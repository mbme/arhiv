import { ComponentChildren } from 'preact';
import { Callback, cx } from '../../scripts/utils';
import { Icon, IconVariant } from './Icon';

type ButtonProps = {
  variant: 'simple' | 'prime' | 'text';
  className?: string;
  onClick?: Callback;
  disabled?: boolean;
  children: ComponentChildren;
  title?: string;
  busy?: boolean;
  alarming?: boolean;
  icon?: IconVariant;
  type?: HTMLButtonElement['type'];
};
export function Button({
  variant,
  className = '',
  onClick,
  disabled,
  children,
  title,
  busy,
  alarming,
  icon,
  type = 'button',
}: ButtonProps) {
  return (
    <button
      type={type}
      className={cx(className, {
        'btn btn-simple': variant === 'simple',
        'btn btn-prime': variant === 'prime',
        'btn btn-text': variant === 'text',
        'btn-alarming': alarming,
        'is-busy': busy,
      })}
      onClick={onClick}
      disabled={disabled || busy}
      title={title}
    >
      {icon && <Icon variant={icon} className="mr-1" />}
      {busy && <Icon variant="spinner" className="mr-2" />}

      {children}
    </button>
  );
}
