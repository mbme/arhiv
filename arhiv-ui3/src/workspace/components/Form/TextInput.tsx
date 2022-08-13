import { useFormField } from './Form';

type TextInputProps = {
  name: string;
  initialValue: string;
  readonly: boolean;
  mandatory: boolean;
};
export function TextInput({ name, initialValue, readonly, mandatory }: TextInputProps) {
  const controlRef = useFormField<HTMLInputElement>(name, (input) => input.value);

  return (
    <input
      ref={controlRef}
      type="text"
      name={name}
      className="field"
      readOnly={readonly}
      required={mandatory}
      defaultValue={initialValue}
    />
  );
}
