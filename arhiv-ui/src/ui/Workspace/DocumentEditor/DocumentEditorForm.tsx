import { useState } from 'preact/hooks';
import { cx, JSONObj } from 'utils';
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
import { Select } from 'components/Form/Select';
import { PreventImplicitSubmissionOnEnter } from 'components/Form/PreventImplicitSubmissionOnEnter';
import { DocumentEditorField } from './DocumentEditorField';
import { useCardLock } from '../workspace-reducer';

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
    collections: DocumentId[]
  ) => Promise<SaveDocumentErrors | void>;
  onDirty?: () => void;
  formRef?: JSXRef<HTMLFormElement>;
};

export function DocumentEditorForm({
  documentId,
  documentType,
  subtype: initialSubtype,
  data: initialData,
  collections: initialCollections,
  onSubmit,
  onDirty,
  formRef,
  autofocus = false,
}: DocumentEditorFormProps) {
  const [isDirty, setDirty] = useState(false);

  useUnsavedChangesWarning(isDirty);
  useCardLock(isDirty);

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

  const onChange = () => {
    if (isDirty) {
      return;
    }

    setDirty(true);
    onDirty?.();
  };

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

  const fieldToFocus = autofocus
    ? fields.find((field) => ignoreReadonly || !field.readonly)
    : undefined;

  return (
    <>
      <form
        className="grid grid-cols-2 mb-8"
        onChange={onChange}
        hidden={!showCollectionPicker && !showSubtypeSelect}
      >
        <PreventImplicitSubmissionOnEnter />

        <label className={cx(showCollectionPicker || 'invisible')}>
          <RefInput
            className="field"
            name="collections"
            documentTypes={collectionTypes}
            defaultValue={collections}
            multiple
            readonly={!canAddCollections}
            onChange={setCollections}
          />
        </label>

        <label className={cx('justify-self-end', showSubtypeSelect || 'invisible')}>
          Subtype &nbsp;
          <Select
            className="field"
            name="subtypes"
            readonly={!canChooseSubtype}
            initialValue={subtype}
            options={subtypes}
            onChange={(value) => setSubtype(value as DocumentSubtype)}
          />
        </label>
      </form>

      <Form onSubmit={submitDocument} formRef={formRef} onChange={onChange}>
        <PreventImplicitSubmissionOnEnter />

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
              autofocus={field === fieldToFocus}
              ignoreReadonly={ignoreReadonly}
              initialValue={initialData[field.name]}
              disabled={!isFieldActive(field, subtype)}
              errors={fieldErrors[field.name]}
            />
          ))}
        </div>
      </Form>
    </>
  );
}
