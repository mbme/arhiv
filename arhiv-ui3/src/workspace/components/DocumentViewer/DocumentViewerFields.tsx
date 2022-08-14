import { DocumentData } from '../../dto';
import { getFieldDescriptions } from '../../schema';
import { DocumentViewerField } from './DocumentViewerField';

type DocumentViewerFieldsProps = {
  documentType: string;
  subtype: string;
  data: DocumentData;
};

export function DocumentViewerFields({ documentType, subtype, data }: DocumentViewerFieldsProps) {
  const fields = getFieldDescriptions(documentType, subtype);

  return (
    <>
      {fields.map((field) => (
        <DocumentViewerField key={field.name} field={field} value={data[field.name]} />
      ))}
    </>
  );
}
