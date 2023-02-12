import { forwardRef, useRef } from 'preact/compat';
import { Callback, cx } from 'utils';
import { JSXChildren } from 'utils/jsx';
import { Icon, IconVariant } from 'components/Icon';

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
  size?: 'sm' | 'md';
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
  size = 'md',
}: ButtonProps) {
  const ref = useRef<HTMLButtonElement>(null);

  const el = ref.current;
  if (el) {
    if (busy && el.style.width.length === 0) {
      el.style.width = `${el.offsetWidth}px`;
    }

    if (!busy && el.style.width.length !== 0) {
      el.style.width = '';
    }
  }

  return (
    <button
      ref={ref}
      type={type}
      className={cx(className, {
        'btn btn-simple': variant === 'simple',
        'btn btn-primary': variant === 'primary',
        'btn btn-text': variant === 'text',
        'btn-alarming': alarming,
        'is-busy': busy,
        'is-sm': size === 'sm',
      })}
      onClick={onClick}
      disabled={disabled || busy}
      title={title}
    >
      {busy ? (
        <Icon variant="spinner" />
      ) : (
        <>
          {leadingIcon && <Icon variant={leadingIcon} />}

          {children}

          {trailingIcon && <Icon variant={trailingIcon} />}
        </>
      )}
    </button>
  );
}

type IconButtonProps = {
  className?: string;
  onClick?: Callback;
  icon: IconVariant;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  title?: string;
};
export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(
  ({ className, icon, onClick, size = 'md', title }, ref) => (
    <button
      ref={ref}
      type="button"
      className={cx(className, 'icon-btn', {
        'p-1': size === 'sm',
        'p-3': size === 'md',
        'p-2': size === 'lg',
        'p-4': size === 'xl',
      })}
      onClick={onClick}
      title={title}
    >
      <Icon
        variant={icon}
        className={cx({
          'h-4 w-4': size === 'sm',
          'h-6 w-6': size === 'lg',
          'h-7 w-7': size === 'xl',
        })}
      />
    </button>
  )
);
