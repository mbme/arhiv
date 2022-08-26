import { useFormField } from './Form';

function parseRefsList(refs: string): string[] | null {
  if (refs.trim().length === 0) {
    return null;
  }

  return refs
    .replaceAll(',', ' ')
    .split(' ')
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

type RefListInputProps = {
  name: string;
  initialValue?: string[];
  readonly: boolean;
  mandatory: boolean;
};
export function RefListInput({ name, initialValue, readonly, mandatory }: RefListInputProps) {
  const controlRef = useFormField<HTMLInputElement>(name, (input) => parseRefsList(input.value));

  return (
    <input
      ref={controlRef}
      type="text"
      name={name}
      className="field"
      readOnly={readonly}
      required={mandatory}
      defaultValue={initialValue?.join(', ') ?? undefined}
    />
  );
}
