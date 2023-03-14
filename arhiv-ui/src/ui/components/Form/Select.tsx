import { useEffect, useRef } from 'react';
import { cx, setElementAttribute } from 'utils';

type Props = {
  id?: string;
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
  id,
  className,
  name,
  initialValue,
  options,
  readonly,
  required,
  disabled,
  onChange,
}: Props) {
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
      id={id}
      className={cx(readonly && 'pointer-events-none', className)}
      name={name}
      disabled={disabled}
      required={required}
      defaultValue={initialValue}
      onChange={(e) => {
        onChange?.(e.currentTarget.value);
      }}
    >
      {options.map((option) => (
        <option key={option} value={option} disabled={readonly}>
          {option}
        </option>
      ))}
    </select>
  );
}
