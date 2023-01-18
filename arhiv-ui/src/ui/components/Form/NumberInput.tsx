import { useEffect, useRef } from 'preact/hooks';

type Props = {
  className?: string;
  name?: string;
  placeholder?: string;
  initialValue?: number;
  min?: number;
  max?: number;
  step?: number;
  readonly?: boolean;
  required?: boolean;
  disabled?: boolean;
};

export function NumberInput({
  className,
  name,
  placeholder,
  initialValue,
  min,
  max,
  step,
  readonly,
  required,
  disabled,
}: Props) {
  const ref = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) {
      throw new Error('element ref must be present');
    }

    el.dataset['value'] = JSON.stringify(initialValue ?? null);
  }, []);

  return (
    <input
      ref={ref}
      className={className}
      name={name}
      type="number"
      onChange={(e) => {
        const el = e.currentTarget;

        const value = el.value ? Number.parseInt(el.value, 10) : null;

        el.dataset['value'] = JSON.stringify(value);
      }}
      defaultValue={initialValue?.toString() ?? undefined}
      min={min}
      max={max}
      step={step}
      placeholder={placeholder}
      required={required}
      disabled={disabled}
      readOnly={readonly}
    />
  );
}
