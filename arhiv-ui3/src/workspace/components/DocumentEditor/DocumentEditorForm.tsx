import { MutableRef, useState } from 'preact/hooks';
import { Callback, JSONObj } from '../../../scripts/utils';
import { DocumentData, DocumentFieldErrors } from '../../dto';
import { RPC } from '../../rpc';
import { getFieldDescriptions } from '../../schema';
import { Form } from '../Form/Form';
import { DocumentEditorFields } from './DocumentEditorFields';
import { DocumentEditorSubtypeSelect, SUBTYPE_FIELD_NAME } from './DocumentEditorSubtypeSelect';

type DocumentEditorFormProps = {
  documentId: string;
  documentType: string;
  subtype: string;
  data: DocumentData;
  onSave: Callback;
  formRef?: MutableRef<HTMLFormElement | null>;
};

export function DocumentEditorForm({
  documentId,
  documentType,
  subtype,
  data,
  onSave,
  formRef,
}: DocumentEditorFormProps) {
  const [documentErrors, setDocumentErrors] = useState<string[]>([]);
  const [fieldErrors, setFieldErrors] = useState<DocumentFieldErrors>({});

  const submitDocument = async (values: JSONObj) => {
    const subtype = values[SUBTYPE_FIELD_NAME] as string;

    const data: JSONObj = {};
    for (const field of getFieldDescriptions(documentType, subtype)) {
      data[field.name] = values[field.name];
    }

    const submitResult = await RPC.SaveDocument({ id: documentId, subtype, data });

    if (submitResult.errors) {
      setDocumentErrors(submitResult.errors.documentErrors);
      setFieldErrors(submitResult.errors.fieldErrors);
    } else {
      setDocumentErrors([]);
      setFieldErrors({});

      onSave();
    }
  };

  return (
    <Form onSubmit={submitDocument} formRef={formRef}>
      <DocumentEditorSubtypeSelect documentType={documentType} initialValue={subtype} />

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
