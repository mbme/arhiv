type DocumentEditorSubtypeSelectProps = {
  subtypes: string[];
  value: string;
  onChange: (value: string) => void;
};

export function DocumentEditorSubtypeSelect({
  subtypes,
  value,
  onChange,
}: DocumentEditorSubtypeSelectProps) {
  return (
    <label className="flex justify-end items-center gap-2">
      <select onChange={(e) => onChange(e.currentTarget.value)} form="">
        {subtypes.map((subtype) => (
          <option key={subtype} value={subtype} selected={subtype === value}>
            {subtype}
          </option>
        ))}
      </select>
    </label>
  );
}
