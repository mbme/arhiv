import { Callback, cx } from 'utils';

type Props = {
  label: string;
  size?: 'sm' | 'md';
  checked?: boolean;
  onClick?: Callback;
  className?: string;
};
export function Badge({ label, checked, onClick, size = 'md', className }: Props) {
  return (
    <span
      className={cx(
        'font-medium px-2.5 py-0.5 rounded-full border select-none',
        {
          'bg-blue-100 text-blue-800 border-blue-400': checked,
          'text-xs': size === 'sm',
          'text-sm': size === 'md',
          'cursor-pointer hover:bg-blue-100/40': Boolean(onClick),
        },
        className,
      )}
      onClick={onClick}
    >
      {label}
    </span>
  );
}
