import { DocumentSubtype } from 'dto';

type DocumentEditorSubtypeSelectProps = {
  subtypes: DocumentSubtype[];
  value: DocumentSubtype;
  onChange: (value: DocumentSubtype) => void;
};

export function DocumentEditorSubtypeSelect({
  subtypes,
  value,
  onChange,
}: DocumentEditorSubtypeSelectProps) {
  return (
    <label className="flex justify-end items-center gap-2">
      <select onChange={(e) => onChange(e.currentTarget.value as DocumentSubtype)} form="">
        {subtypes.map((subtype) => (
          <option key={subtype} value={subtype} selected={subtype === value}>
            {subtype}
          </option>
        ))}
      </select>
    </label>
  );
}
