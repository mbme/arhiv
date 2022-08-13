import { DocumentData } from '../../dto';
import { getFieldDescriptions, isFieldActive } from '../../schema';
import { DocumentEditorField } from './DocumentEditorField';

type DocumentEditorFieldsProps = {
  documentType: string;
  subtype: string;
  data: DocumentData;
};

export function DocumentEditorFields({ documentType, subtype, data }: DocumentEditorFieldsProps) {
  const fields = getFieldDescriptions(documentType);

  return (
    <>
      {fields.map((field) => (
        <DocumentEditorField
          key={field.name}
          field={field}
          initialValue={data[field.name]}
          hidden={!isFieldActive(field, subtype)}
        />
      ))}
    </>
  );
}
