import { useRef, useState } from 'react';
import { shallowEqualArrays } from 'shallow-equal';
import { cx, JSONObj } from 'utils';
import {
  DocumentData,
  DocumentFieldErrors,
  DocumentId,
  DocumentType,
  SaveDocumentErrors,
} from 'dto';
import { getCollectionTypesForDocument, getDataDescription } from 'utils/schema';
import { JSXRef, mergeRefs } from 'utils/jsx';
import { useUpdateEffect } from 'utils/hooks';
import { CollectionPicker } from 'components/CollectionPicker';
import { Form, markFormDirty } from 'components/Form/Form';
import { PreventImplicitSubmissionOnEnter } from 'components/Form/PreventImplicitSubmissionOnEnter';
import { ErrorMessage } from 'components/ErrorMessage';
import { DocumentField } from './DocumentField';

type DocumentEditorFormProps = {
  autofocus?: boolean;
  documentType: DocumentType;
  data: DocumentData;
  collections: DocumentId[];
  onSubmit: (data: JSONObj, collections: DocumentId[]) => Promise<SaveDocumentErrors | undefined>;
  formRef?: JSXRef<HTMLFormElement>;
  readonlyOverride?: boolean;
};

export function DocumentEditor({
  documentType,
  data: initialData,
  collections: initialCollections,
  onSubmit,
  formRef: outerFormRef,
  autofocus = false,
  readonlyOverride,
}: DocumentEditorFormProps) {
  const formRef = useRef<HTMLFormElement | null>(null);

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

  const fields = getDataDescription(documentType).fields;

  const fieldToFocus = autofocus
    ? fields.find((field) => readonlyOverride === false || !field.readonly)
    : undefined;

  useUpdateEffect(() => {
    if (formRef.current && !shallowEqualArrays(collections, initialCollections)) {
      markFormDirty(formRef.current, true);
    }
  }, [collections, initialCollections]);

  return (
    <Form
      onSubmit={submitDocument}
      onReset={() => {
        setCollections(initialCollections);
      }}
      formRef={mergeRefs(formRef, outerFormRef)}
    >
      <PreventImplicitSubmissionOnEnter />

      <label className={cx('inline-block mb-6', showCollectionPicker || 'invisible')}>
        {hasCollections && <h1 className="section-heading">Collections:</h1>}
        {showCollectionPicker && (
          <CollectionPicker
            collectionTypes={collectionTypes}
            ids={collections}
            onChange={setCollections}
          />
        )}
      </label>

      {documentErrors.map((error, index) => (
        <ErrorMessage key={index} className="pl-1 my-2">
          {error}
        </ErrorMessage>
      ))}

      <div className="divide-y divide-dashed border-gray-200">
        {fields.map((field) => (
          <DocumentField
            key={field.name}
            field={field}
            autofocus={field === fieldToFocus}
            initialValue={initialData[field.name]}
            errors={fieldErrors[field.name]}
            readonly={readonlyOverride ?? field.readonly}
          />
        ))}
      </div>
    </Form>
  );
}
