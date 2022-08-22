import { MutableRef, useState } from 'preact/hooks';
import { JSONObj } from '../../../scripts/utils';
import { DocumentData, DocumentFieldErrors } from '../../dto';
import { RPC } from '../../rpc';
import { getDefaultSubtype, getFieldDescriptions } from '../../schema';
import { Form } from '../Form/Form';
import { DocumentEditorFields } from './DocumentEditorFields';
import { DocumentEditorSubtypeSelect } from './DocumentEditorSubtypeSelect';

type DocumentEditorFormProps = {
  documentId?: string;
  documentType: string;
  subtype?: string;
  data?: DocumentData;
  onSave: (id: string) => void;
  formRef?: MutableRef<HTMLFormElement | null>;
};

export function DocumentEditorForm({
  documentId,
  documentType,
  subtype: initialSubtype,
  data = {},
  onSave,
  formRef,
}: DocumentEditorFormProps) {
  const [documentErrors, setDocumentErrors] = useState<string[]>([]);
  const [fieldErrors, setFieldErrors] = useState<DocumentFieldErrors>({});
  const [subtype, setSubtype] = useState(initialSubtype ?? getDefaultSubtype(documentType));

  const submitDocument = async (values: JSONObj) => {
    const data: JSONObj = {};
    for (const field of getFieldDescriptions(documentType, subtype)) {
      data[field.name] = values[field.name];
    }

    const submitResult = await (documentId
      ? RPC.SaveDocument({ id: documentId, subtype, data })
      : RPC.CreateDocument({ documentType, subtype, data }));

    if (submitResult.errors) {
      setDocumentErrors(submitResult.errors.documentErrors);
      setFieldErrors(submitResult.errors.fieldErrors);
    } else {
      setDocumentErrors([]);
      setFieldErrors({});

      onSave(submitResult.typeName === 'CreateDocument' ? submitResult.id! : documentId!);
    }
  };

  return (
    <Form onSubmit={submitDocument} formRef={formRef}>
      <DocumentEditorSubtypeSelect
        documentType={documentType}
        value={subtype}
        onChange={setSubtype}
      />

      {documentErrors.map((error, index) => (
        <div key={index} className="text-red-500 text-xl pl-1 my-2">
          {error}
        </div>
      ))}

      <DocumentEditorFields
        documentType={documentType}
        subtype={subtype}
        data={data}
        errors={fieldErrors}
      />
    </Form>
  );
}
