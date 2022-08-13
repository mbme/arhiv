import { useFormField } from './Form';

type SelectProps = {
  name: string;
  initialValue?: string;
  options: string[];
  readonly: boolean;
  mandatory: boolean;
};
export function Select({ name, initialValue, options, readonly, mandatory }: SelectProps) {
  const controlRef = useFormField<HTMLSelectElement>(name, (select) => select.value);

  return (
    <select
      ref={controlRef}
      name={name}
      className="field"
      readOnly={readonly}
      value={initialValue ?? undefined}
    >
      {mandatory || <option key="" value="" />}

      {options.map((option) => (
        <option key={option} value={option}>
          {option}
        </option>
      ))}
    </select>
  );
}
