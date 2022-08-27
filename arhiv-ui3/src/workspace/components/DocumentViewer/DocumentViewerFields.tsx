import { DocumentData } from '../../dto';
import { getFieldDescriptions, isAttachment } from '../../schema';
import { AttachmentPreview } from './AttachmentPreview';
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
        class="my-16 mx-auto"
      />
    );
  }

  const fields = getFieldDescriptions(documentType, subtype);

  return (
    <>
      {isAttachment(documentType) && (
        <div className="mb-8">
          <AttachmentPreview subtype={subtype} data={data} />
        </div>
      )}

      {fields.map((field) => (
        <DocumentViewerField key={field.name} field={field} value={data[field.name]} />
      ))}
    </>
  );
}
