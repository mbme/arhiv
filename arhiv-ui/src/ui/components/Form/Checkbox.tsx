import { useEffect, useRef } from 'preact/hooks';

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
      onInput={(e) => {
        const el = e.currentTarget;

        el.dataset['value'] = JSON.stringify(el.checked);
      }}
      checked={initialValue}
    />
  );
}
