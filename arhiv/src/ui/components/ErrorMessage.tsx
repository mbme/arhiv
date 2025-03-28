import { cx } from 'utils';

interface Props {
  className?: string;
  small?: boolean;
  children: string;
}

export function ErrorMessage({ className, small, children }: Props) {
  return (
    <div
      className={cx('text-red-500 break-words w-full', className, {
        'text-xs': small,
        'text-xl': !small,
      })}
    >
      {children}
    </div>
  );
}
