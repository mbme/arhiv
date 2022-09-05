import { Callback, cx } from '../../scripts/utils';
import { JSXChildren } from '../types';
import { Icon, IconVariant } from './Icon';

// TODO: button size - small / regular (maybe boolean property)
// TODO: lock button width on busy, and show only spinner

type ButtonProps = {
  variant: 'simple' | 'primary' | 'text';
  className?: string;
  onClick?: Callback;
  disabled?: boolean;
  children: JSXChildren;
  title?: string;
  busy?: boolean;
  alarming?: boolean;
  leadingIcon?: IconVariant;
  trailingIcon?: IconVariant;
  type?: 'button' | 'submit' | 'reset';
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
  leadingIcon,
  trailingIcon,
  type = 'button',
}: ButtonProps) {
  return (
    <button
      type={type}
      className={cx(className, {
        'btn btn-simple': variant === 'simple',
        'btn btn-primary': variant === 'primary',
        'btn btn-text': variant === 'text',
        'btn-alarming': alarming,
        'is-busy': busy,
      })}
      onClick={onClick}
      disabled={disabled || busy}
      title={title}
    >
      {leadingIcon && <Icon variant={leadingIcon} />}

      {children}

      {trailingIcon && <Icon variant={trailingIcon} />}

      {busy && <Icon variant="spinner" />}
    </button>
  );
}

type IconButtonProps = {
  className?: string;
  onClick?: Callback;
  icon: IconVariant;
  size?: 'lg';
};
export function IconButton({ className, icon, onClick, size, ...props }: IconButtonProps) {
  return (
    <button
      type="button"
      className={cx(className, 'icon-btn', {
        'p-4': size === 'lg',
      })}
      onClick={onClick}
      {...props}
    >
      <Icon
        variant={icon}
        className={cx({
          'h-7 w-7': size === 'lg',
        })}
      />
    </button>
  );
}
