import { getDataDescription } from '../../schema';

type DocumentEditorSubtypeSelectProps = {
  documentType: string;
  value: string;
  onChange: (value: string) => void;
};
export function DocumentEditorSubtypeSelect({
  documentType,
  value,
  onChange,
}: DocumentEditorSubtypeSelectProps) {
  const subtypes = getDataDescription(documentType).subtypes || [];

  return (
    <label class="flex justify-end items-center gap-2 mb-8" hidden={subtypes.length < 2}>
      <select
        className="field"
        name="@subtype"
        onChange={(e) => onChange((e.target as HTMLSelectElement).value)}
      >
        {subtypes.map((subtype) => (
          <option key={subtype} value={subtype} selected={subtype === value}>
            {subtype}
          </option>
        ))}
      </select>
    </label>
  );
}
