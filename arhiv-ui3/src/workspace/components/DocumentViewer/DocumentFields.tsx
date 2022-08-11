import { DataDescription, DocumentData } from '../../dto';
import { DocumentField } from './DocumentField';

type DocumentFieldsProps = {
  data: DocumentData;
  dataDescription: DataDescription;
  subtype: string;
};

export function DocumentFields({ data, dataDescription, subtype }: DocumentFieldsProps) {
  const fields = dataDescription.fields.filter(
    (field) => field.for_subtypes?.includes(subtype) ?? true
  );

  return (
    <>
      {fields.map((field) => (
        <DocumentField key={field.name} field={field} value={data[field.name]} />
      ))}
    </>
  );
}
