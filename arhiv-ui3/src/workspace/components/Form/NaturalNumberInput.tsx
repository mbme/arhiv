import { useFormField } from './Form';

type NaturalNumberInputProps = {
  name: string;
  initialValue?: number;
  readonly: boolean;
  mandatory: boolean;
};
export function NaturalNumberInput({
  name,
  initialValue,
  readonly,
  mandatory,
}: NaturalNumberInputProps) {
  const controlRef = useFormField<HTMLInputElement>(name, (input) =>
    input.value ? Number.parseInt(input.value, 10) : undefined
  );

  return (
    <input
      ref={controlRef}
      type="number"
      min={0}
      step={1}
      name={name}
      className="field"
      readOnly={readonly}
      required={mandatory}
      defaultValue={initialValue?.toString() ?? undefined}
    />
  );
}
