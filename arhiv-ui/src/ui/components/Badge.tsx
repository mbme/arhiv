import { Callback, cx } from 'utils';

type Props = {
  label: string;
  size?: 'sm' | 'md';
  checked?: boolean;
  onClick?: Callback;
};
export function Badge({ label, checked, onClick, size = 'md' }: Props) {
  return (
    <span
      className={cx(
        'font-medium px-2.5 py-0.5 rounded-full border cursor-pointer select-none hover:bg-blue-100/40',
        {
          'bg-blue-100 text-blue-800 border-blue-400': checked,
          'text-xs': size === 'sm',
          'text-sm': size === 'md',
        }
      )}
      onClick={onClick}
    >
      {label}
    </span>
  );
}
