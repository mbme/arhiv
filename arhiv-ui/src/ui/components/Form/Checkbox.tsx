import { useRef } from 'preact/hooks';

type Props = {
  className?: string;
  name?: string;
  initialValue?: boolean;
  readonly?: boolean;
  required?: boolean;
  disabled?: boolean;
};

export function Checkbox({ className, name, initialValue, readonly, required, disabled }: Props) {
  const ref = useRef<HTMLInputElement>(null);

  return (
    <input
      ref={ref}
      className={className}
      name={name}
      type="checkbox"
      required={required}
      disabled={disabled}
      readOnly={readonly}
      onInputCapture={(e) => {
        if (readonly) {
          e.currentTarget.checked = !e.currentTarget.checked;
          e.preventDefault();
        }
      }}
      checked={initialValue}
    />
  );
}
