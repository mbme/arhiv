import { useState } from 'preact/hooks';
import { JSONObj } from 'utils';
import {
  DocumentData,
  DocumentFieldErrors,
  DocumentId,
  DocumentType,
  DocumentSubtype,
  SaveDocumentErrors,
} from 'dto';
import { useUnsavedChangesWarning } from 'utils/hooks';
import {
  getCollectionTypesForDocument,
  getDataDescription,
  getDefaultSubtype,
  getFieldDescriptions,
  isFieldActive,
} from 'utils/schema';
import { JSXRef } from 'utils/jsx';
import { Form } from 'components/Form/Form';
import { RefInput } from 'components/Form/RefInput';
import { PreventImplicitSubmissionOnEnter } from 'components/Form/PreventImplicitSubmissionOnEnter';
import { DocumentEditorField } from './DocumentEditorField';
import { DocumentEditorSubtypeSelect } from './DocumentEditorSubtypeSelect';
import { useCardLock, useCardContext } from '../workspace-reducer';

type DocumentEditorFormProps = {
  documentId?: string;
  documentType: DocumentType;
  subtype: DocumentSubtype;
  data: DocumentData;
  collections?: DocumentId[];
  onSubmit: (
    data: JSONObj,
    subtype: DocumentSubtype,
    collections: DocumentId[]
  ) => Promise<SaveDocumentErrors | void>;
  formRef?: JSXRef<HTMLFormElement>;
};

export function DocumentEditorForm({
  documentId,
  documentType,
  subtype: initialSubtype,
  data: initialData,
  collections: initialCollections,
  onSubmit,
  formRef,
}: DocumentEditorFormProps) {
  const { openDocument } = useCardContext();

  useUnsavedChangesWarning();
  useCardLock();

  const [documentErrors, setDocumentErrors] = useState<string[]>([]);
  const [fieldErrors, setFieldErrors] = useState<DocumentFieldErrors>({});
  const [subtype, setSubtype] = useState(initialSubtype ?? getDefaultSubtype(documentType));
  const [collections, setCollections] = useState(initialCollections ?? []);

  const collectionTypes = getCollectionTypesForDocument(documentType);
  const subtypes = getDataDescription(documentType).subtypes || [];

  const canAddCollections = collectionTypes.length > 0;
  const canChooseSubtype = subtypes.length > 1;

  const submitDocument = async (data: JSONObj) => {
    const errors = await onSubmit(data, subtype, collections);

    if (errors) {
      setDocumentErrors(errors.documentErrors);
      setFieldErrors(errors.fieldErrors);
    } else {
      setDocumentErrors([]);
      setFieldErrors({});
    }
  };

  const ignoreReadonly = !documentId;
  const fields = getFieldDescriptions(documentType);

  const fieldToFocus = fields.find((field) => ignoreReadonly || !field.readonly);

  return (
    <div>
      <div className="flex justify-between mb-8" hidden={!canAddCollections && !canChooseSubtype}>
        <RefInput
          className="field"
          name="collections"
          documentTypes={collectionTypes}
          defaultValue={collections}
          multiple
          readonly={!canAddCollections}
          onChange={setCollections}
          onRefClick={openDocument}
        />

        {canChooseSubtype && (
          <DocumentEditorSubtypeSelect subtypes={subtypes} value={subtype} onChange={setSubtype} />
        )}
      </div>

      {documentErrors.map((error, index) => (
        <div key={index} className="text-red-500 text-xl pl-1 my-2">
          {error}
        </div>
      ))}

      <Form onSubmit={submitDocument} formRef={formRef}>
        <PreventImplicitSubmissionOnEnter />

        <div className="divide-y divide-dashed">
          {fields.map((field) => (
            <DocumentEditorField
              key={field.name}
              field={field}
              autofocus={field === fieldToFocus}
              ignoreReadonly={ignoreReadonly}
              initialValue={initialData[field.name]}
              disabled={!isFieldActive(field, subtype)}
              errors={fieldErrors[field.name]}
            />
          ))}
        </div>
      </Form>
    </div>
  );
}
