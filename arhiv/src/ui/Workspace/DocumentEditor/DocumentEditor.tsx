import { useState } from 'react';
import { cx, JSONObj } from 'utils';
import {
  DocumentData,
  DocumentFieldErrors,
  DocumentId,
  DocumentType,
  SaveDocumentErrors,
} from 'dto';
import { getCollectionTypesForDocument, getDataDescription } from 'utils/schema';
import { JSXRef } from 'utils/jsx';
import { CollectionPicker } from 'components/CollectionPicker';
import { Form } from 'components/Form/Form';
import { PreventImplicitSubmissionOnEnter } from 'components/Form/PreventImplicitSubmissionOnEnter';
import { DocumentField } from './DocumentField';

type DocumentEditorFormProps = {
  autofocus?: boolean;
  documentId?: string;
  documentType: DocumentType;
  data: DocumentData;
  collections: DocumentId[];
  onSubmit: (data: JSONObj, collections: DocumentId[]) => Promise<SaveDocumentErrors | undefined>;
  formRef?: JSXRef<HTMLFormElement>;
};

export function DocumentEditor({
  documentId,
  documentType,
  data: initialData,
  collections: initialCollections,
  onSubmit,
  formRef,
  autofocus = false,
}: DocumentEditorFormProps) {
  const [documentErrors, setDocumentErrors] = useState<string[]>([]);
  const [fieldErrors, setFieldErrors] = useState<DocumentFieldErrors>({});
  const [collections, setCollections] = useState(initialCollections);

  const collectionTypes = getCollectionTypesForDocument(documentType);

  const hasCollections = collections.length > 0;
  const canAddCollections = collectionTypes.length > 0;

  const showCollectionPicker = hasCollections || canAddCollections;

  const submitDocument = async (data: JSONObj) => {
    const errors = await onSubmit(data, collections).catch((e: unknown) => ({
      documentErrors: [String(e)],
      fieldErrors: {},
    }));

    if (errors) {
      setDocumentErrors(errors.documentErrors);
      setFieldErrors(errors.fieldErrors);
    } else {
      setDocumentErrors([]);
      setFieldErrors({});
    }
  };

  const ignoreReadonly = !documentId;
  const fields = getDataDescription(documentType).fields;

  const fieldToFocus = autofocus
    ? fields.find((field) => ignoreReadonly || !field.readonly)
    : undefined;

  return (
    <Form onSubmit={submitDocument} formRef={formRef}>
      <PreventImplicitSubmissionOnEnter />

      <label className={cx(showCollectionPicker || 'invisible')}>
        {showCollectionPicker && (
          <CollectionPicker
            collectionTypes={collectionTypes}
            ids={collections}
            onChange={setCollections}
          />
        )}
      </label>

      {documentErrors.map((error, index) => (
        <div key={index} className="text-red-500 text-xl pl-1 my-2">
          {error}
        </div>
      ))}

      <div className="divide-y divide-dashed">
        {fields.map((field) => (
          <DocumentField
            key={field.name}
            field={field}
            autofocus={field === fieldToFocus}
            ignoreReadonly={ignoreReadonly}
            initialValue={initialData[field.name]}
            errors={fieldErrors[field.name]}
          />
        ))}
      </div>
    </Form>
  );
}
