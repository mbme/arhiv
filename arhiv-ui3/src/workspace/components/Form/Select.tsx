import { Ref } from 'preact';

type Props = {
  className?: string;
  name?: string;
  initialValue?: string;
  options: string[];
  readonly?: boolean;
  required?: boolean;
  disabled?: boolean;
  innerRef?: Ref<HTMLSelectElement>;
};

export function Select({
  className,
  name,
  initialValue,
  options,
  readonly,
  required,
  disabled,
  innerRef,
}: Props) {
  if (options.includes('')) {
    throw new Error('options must not include empty string');
  }

  return (
    <select
      ref={innerRef}
      className={className}
      name={name}
      disabled={disabled}
      readonly={readonly}
      required={required}
    >
      <option key="" value="" disabled={readonly} />

      {options.map((option) => (
        <option key={option} value={option} disabled={readonly} selected={option === initialValue}>
          {option}
        </option>
      ))}
    </select>
  );
}
