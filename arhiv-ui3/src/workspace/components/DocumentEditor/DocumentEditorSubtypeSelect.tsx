import { getDataDescription } from '../../schema';
import { Select } from '../Form';

export const SUBTYPE_FIELD_NAME = '@subtype';

type DocumentEditorSubtypeSelectProps = {
  documentType: string;
  initialValue: string;
};
export function DocumentEditorSubtypeSelect({
  documentType,
  initialValue,
}: DocumentEditorSubtypeSelectProps) {
  const subtypes = getDataDescription(documentType).subtypes || [];

  return (
    <label class="flex justify-end items-center gap-2 mb-8" hidden={subtypes.length < 2}>
      <Select
        name={SUBTYPE_FIELD_NAME}
        options={subtypes}
        initialValue={initialValue}
        readonly={false}
        mandatory={false}
      />
    </label>
  );
}
