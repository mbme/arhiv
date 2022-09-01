import { Ref } from 'preact';

type Props = {
  className?: string;
  name?: string;
  initialValue?: boolean;
  readonly?: boolean;
  required?: boolean;
  disabled?: boolean;
  innerRef?: Ref<HTMLInputElement>;
};

export function Checkbox({
  className,
  name,
  initialValue,
  readonly,
  required,
  disabled,
  innerRef,
}: Props) {
  return (
    <input
      ref={innerRef}
      className={className}
      name={name}
      type="checkbox"
      required={required}
      disabled={disabled}
      readonly={readonly}
      onInputCapture={(e) => {
        if (readonly) {
          e.currentTarget.checked = !e.currentTarget.checked;
          e.preventDefault();
          e.stopImmediatePropagation();
        }
      }}
      checked={initialValue}
    />
  );
}
