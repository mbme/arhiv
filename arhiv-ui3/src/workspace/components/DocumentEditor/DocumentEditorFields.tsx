import { DocumentData, DocumentFieldErrors } from '../../dto';
import { getFieldDescriptions, isFieldActive } from '../../schema';
import { DocumentEditorField } from './DocumentEditorField';

type DocumentEditorFieldsProps = {
  documentType: string;
  subtype: string;
  data: DocumentData;
  errors: DocumentFieldErrors;
};

export function DocumentEditorFields({
  documentType,
  subtype,
  data,
  errors,
}: DocumentEditorFieldsProps) {
  const fields = getFieldDescriptions(documentType);

  return (
    <>
      {fields.map((field) => (
        <DocumentEditorField
          key={field.name}
          field={field}
          initialValue={data[field.name]}
          hidden={!isFieldActive(field, subtype)}
          errors={errors[field.name]}
        />
      ))}
    </>
  );
}
