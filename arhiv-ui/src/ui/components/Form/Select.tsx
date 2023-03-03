import { useEffect, useRef } from 'preact/hooks';
import { cx, setElementAttribute } from 'utils';

type Props = {
  className?: string;
  name?: string;
  initialValue?: string;
  options: string[];
  readonly?: boolean;
  required?: boolean;
  disabled?: boolean;
  onChange?: (value: string) => void;
};

export function Select({
  className,
  name,
  initialValue,
  options,
  readonly,
  required,
  disabled,
  onChange,
}: Props) {
  if (options.includes('')) {
    throw new Error('options must not include an empty string');
  }

  const selectRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (selectRef.current) {
      setElementAttribute(selectRef.current, 'readonly', readonly);
    }
  }, [readonly]);

  return (
    <select
      ref={(el) => {
        selectRef.current = el;

        if (el) {
          setElementAttribute(el, 'readonly', readonly);
        }
      }}
      className={cx(readonly && 'pointer-events-none', className)}
      name={name}
      disabled={disabled}
      required={required}
      defaultValue={initialValue}
      onChange={(e) => {
        onChange?.(e.currentTarget.value);
      }}
    >
      <option key="" value="" disabled={readonly} />

      {options.map((option) => (
        <option key={option} value={option} disabled={readonly}>
          {option}
        </option>
      ))}
    </select>
  );
}
