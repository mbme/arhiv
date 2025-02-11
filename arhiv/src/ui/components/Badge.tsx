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
        'font-medium px-2.5 py-0.5 rounded-full border border-gray-200 select-none',
        {
          'var-item-active-bg-color var-active-color border-blue-400 dark:border-blue-600': checked,
          'text-xs': size === 'sm',
          'text-sm': size === 'md',
          'cursor-pointer hover:var-item-active-bg-color': Boolean(onClick),
        },
        className,
      )}
      onClick={onClick}
    >
      {label}
    </span>
  );
}
