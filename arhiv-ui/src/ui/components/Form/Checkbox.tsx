import { JSXRef } from 'utils/jsx';

type Props = {
  className?: string;
  name?: string;
  initialValue?: boolean;
  readonly?: boolean;
  required?: boolean;
  disabled?: boolean;
  innerRef?: JSXRef<HTMLInputElement>;
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
