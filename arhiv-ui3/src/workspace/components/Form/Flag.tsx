import { useFormField } from './Form';

type FlagProps = {
  name: string;
  initialValue: boolean;
  readonly: boolean;
  mandatory: boolean;
};
export function Flag({ name, initialValue, readonly, mandatory }: FlagProps) {
  const controlRef = useFormField<HTMLInputElement>(name, (input) => input.checked);

  return (
    <input
      ref={controlRef}
      type="checkbox"
      name={name}
      className="field"
      defaultChecked={initialValue}
      readOnly={readonly}
      required={mandatory}
    />
  );
}
