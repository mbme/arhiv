import { DocumentData } from '../../../dto';
import { getFieldDescriptions } from '../../utils/schema';
import { DocumentViewerField } from './DocumentViewerField';

type DocumentViewerFieldsProps = {
  documentType: string;
  subtype: string;
  data: DocumentData;
};

export function DocumentViewerFields({ documentType, subtype, data }: DocumentViewerFieldsProps) {
  if (!documentType) {
    return (
      <img
        src="/public/nothing-to-see-here.jpg"
        alt="funny picture for the erased document"
        className="my-16 mx-auto"
      />
    );
  }

  const fields = getFieldDescriptions(documentType, subtype);

  return (
    <div className="divide-y divide-dashed">
      {fields.map((field) => (
        <DocumentViewerField key={field.name} field={field} value={data[field.name]} />
      ))}
    </div>
  );
}
