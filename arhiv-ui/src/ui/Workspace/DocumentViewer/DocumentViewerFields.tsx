import { DocumentData, DocumentType, DocumentSubtype } from 'dto';
import { getFieldDescriptions } from 'utils/schema';
import { DocumentViewerField, FieldValue } from './DocumentViewerField';

type DocumentViewerFieldsProps = {
  documentType: DocumentType;
  subtype: DocumentSubtype;
  data: DocumentData;
};

export function DocumentViewerFields({ documentType, subtype, data }: DocumentViewerFieldsProps) {
  const fields = getFieldDescriptions(documentType, subtype);

  return (
    <div className="divide-y divide-dashed">
      {fields.map((field) => {
        const value = data[field.name];

        if (!value) {
          return null;
        }

        return (
          <DocumentViewerField key={field.name} name={field.name}>
            <FieldValue field={field} value={value} />
          </DocumentViewerField>
        );
      })}
    </div>
  );
}
