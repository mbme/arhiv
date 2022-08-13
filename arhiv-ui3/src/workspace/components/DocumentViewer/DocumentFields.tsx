import { DocumentData } from '../../dto';
import { DataDescriptionField, getFieldDescriptions, isFieldActive } from '../../schema';
import { DocumentField } from './DocumentField';

type DocumentFieldsProps = {
  documentType: string;
  subtype: string;
  data: DocumentData;
};

export function DocumentFields({ documentType, subtype, data }: DocumentFieldsProps) {
  const fields = getFieldDescriptions(documentType).filter((field) =>
    isFieldActive(field, subtype)
  );

  return (
    <>
      {fields.map((field) => (
        <DocumentField key={field.name} field={field} value={data[field.name]} />
      ))}
    </>
  );
}
