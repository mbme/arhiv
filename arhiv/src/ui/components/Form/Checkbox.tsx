import { useRef } from 'react';

type Props = {
  id?: string;
  className?: string;
  name?: string;
  initialValue?: boolean;
  value?: boolean;
  onChange?: (value: boolean) => void;
  readonly?: boolean;
  required?: boolean;
  disabled?: boolean;
};

export function Checkbox({
  id,
  className,
  name,
  initialValue,
  value,
  onChange,
  readonly,
  required,
  disabled,
}: Props) {
  const ref = useRef<HTMLInputElement>(null);

  return (
    <input
      ref={ref}
      id={id}
      className={className}
      name={name}
      type="checkbox"
      required={required}
      disabled={disabled}
      readOnly={readonly}
      checked={value}
      onInputCapture={(e) => {
        if (readonly) {
          e.currentTarget.checked = !e.currentTarget.checked;
          e.preventDefault();
        }
      }}
      defaultChecked={initialValue}
      onChange={(e) => {
        onChange?.(e.currentTarget.checked);
      }}
    />
  );
}
