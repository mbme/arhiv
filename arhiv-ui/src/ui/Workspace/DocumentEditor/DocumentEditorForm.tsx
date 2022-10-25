import { useState } from 'preact/hooks';
import { JSONObj } from '../../utils';
import { DocumentData, DocumentFieldErrors } from '../../../dto';
import { RPC } from '../../utils/rpc';
import { useUnsavedChangesWarning } from '../../utils/hooks';
import { getDefaultSubtype, getFieldDescriptions, isFieldActive } from '../../utils/schema';
import { JSXRef } from '../../utils/jsx';
import { Form } from '../../components/Form/Form';
import { PreventImplicitSubmissionOnEnter } from '../../components/Form/PreventImplicitSubmissionOnEnter';
import { DocumentEditorField } from './DocumentEditorField';
import { DocumentEditorSubtypeSelect } from './DocumentEditorSubtypeSelect';
import { useCardLock } from '../workspace-reducer';

type DocumentEditorFormProps = {
  documentId?: string;
  documentType: string;
  subtype?: string;
  data?: DocumentData;
  onSave: (id: string) => void;
  formRef?: JSXRef<HTMLFormElement>;
};

export function DocumentEditorForm({
  documentId,
  documentType,
  subtype: initialSubtype,
  data = {},
  onSave,
  formRef,
}: DocumentEditorFormProps) {
  useUnsavedChangesWarning();
  useCardLock();

  const [documentErrors, setDocumentErrors] = useState<string[]>([]);
  const [fieldErrors, setFieldErrors] = useState<DocumentFieldErrors>({});
  const [subtype, setSubtype] = useState(initialSubtype ?? getDefaultSubtype(documentType));

  const submitDocument = async (data: JSONObj) => {
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

  const fields = getFieldDescriptions(documentType);

  return (
    <Form onSubmit={submitDocument} formRef={formRef}>
      <PreventImplicitSubmissionOnEnter />

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

      <div className="divide-y divide-dashed">
        {fields.map((field) => (
          <DocumentEditorField
            key={field.name}
            field={field}
            ignoreReadonly={!documentId}
            initialValue={data[field.name]}
            disabled={!isFieldActive(field, subtype)}
            errors={fieldErrors[field.name]}
          />
        ))}
      </div>
    </Form>
  );
}
