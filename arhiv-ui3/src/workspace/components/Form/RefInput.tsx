import { useFormField } from './Form';

type RefInputProps = {
  name: string;
  initialValue?: string;
  readonly: boolean;
  mandatory: boolean;
};
export function RefInput({ name, initialValue, readonly, mandatory }: RefInputProps) {
  const controlRef = useFormField<HTMLInputElement>(
    name,
    (input) => input.value.trim() || undefined
  );

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
