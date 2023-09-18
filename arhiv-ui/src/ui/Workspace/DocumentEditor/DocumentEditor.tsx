import { useState } from 'react';
import { cx, JSONObj } from 'utils';
import {
  DocumentData,
  DocumentFieldErrors,
  DocumentId,
  DocumentType,
  DocumentSubtype,
  SaveDocumentErrors,
} from 'dto';
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
import { Select } from 'components/Form/Select';
import { PreventImplicitSubmissionOnEnter } from 'components/Form/PreventImplicitSubmissionOnEnter';
import { DocumentField } from './DocumentField';

type DocumentEditorFormProps = {
  autofocus?: boolean;
  documentId?: string;
  documentType: DocumentType;
  subtype: DocumentSubtype;
  data: DocumentData;
  collections?: DocumentId[];
  onSubmit: (
    data: JSONObj,
    subtype: DocumentSubtype,
    collections: DocumentId[],
  ) => Promise<SaveDocumentErrors | void>;
  formRef?: JSXRef<HTMLFormElement>;
};

export function DocumentEditor({
  documentId,
  documentType,
  subtype: initialSubtype,
  data: initialData,
  collections: initialCollections,
  onSubmit,
  formRef,
  autofocus = false,
}: DocumentEditorFormProps) {
  const [documentErrors, setDocumentErrors] = useState<string[]>([]);
  const [fieldErrors, setFieldErrors] = useState<DocumentFieldErrors>({});
  const [subtype, setSubtype] = useState(initialSubtype ?? getDefaultSubtype(documentType));
  const [collections, setCollections] = useState(initialCollections ?? []);

  const collectionTypes = getCollectionTypesForDocument(documentType);
  const subtypes = getDataDescription(documentType).subtypes || [];

  const hasCollections = collections.length > 0;
  const canAddCollections = collectionTypes.length > 0;
  const canChooseSubtype = subtypes.length > 1;

  const showCollectionPicker = hasCollections || canAddCollections;
  const showSubtypeSelect = canChooseSubtype;

  const submitDocument = async (data: JSONObj) => {
    delete data['@collections'];
    delete data['@subtypes'];

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

  const fieldToFocus = autofocus
    ? fields.find((field) => ignoreReadonly || !field.readonly)
    : undefined;

  return (
    <Form onSubmit={submitDocument} formRef={formRef}>
      <PreventImplicitSubmissionOnEnter />

      <div className="grid grid-cols-2 mb-8" hidden={!showCollectionPicker && !showSubtypeSelect}>
        <label className={cx(showCollectionPicker || 'invisible')}>
          <RefInput
            className="field"
            name="@collections"
            documentTypes={collectionTypes}
            defaultValue={initialCollections}
            multiple
            readonly={!canAddCollections}
            onChange={setCollections}
          />
        </label>

        <label className={cx('justify-self-end', showSubtypeSelect || 'invisible')}>
          Subtype &nbsp;
          <Select
            className="field"
            name="@subtypes"
            readonly={!canChooseSubtype}
            initialValue={subtype}
            options={subtypes}
            onChange={(value) => setSubtype(value as DocumentSubtype)}
          />
        </label>
      </div>

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
            disabled={!isFieldActive(field, subtype)}
            errors={fieldErrors[field.name]}
          />
        ))}
      </div>
    </Form>
  );
}
