import { DocumentData } from '../../dto';
import { DataDescriptionField } from '../../schema';
import { DocumentField } from './DocumentField';

type DocumentFieldsProps = {
  data: DocumentData;
  fields: DataDescriptionField[];
};

export function DocumentFields({ data, fields }: DocumentFieldsProps) {
  return (
    <>
      {fields.map((field) => (
        <DocumentField key={field.name} field={field} value={data[field.name]} />
      ))}
    </>
  );
}
