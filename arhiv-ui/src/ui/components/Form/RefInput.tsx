import { useEffect, useRef, useState } from 'react';
import { cx } from 'utils';
import { useLatestRef } from 'utils/hooks';
import { DocumentId, DocumentType } from 'dto';
import { Ref, useDocuments } from 'components/Ref';
import { DocumentPicker } from 'components/DocumentPicker';
import { Button, IconButton } from 'components/Button';
import { QueryError } from 'components/QueryError';
import { HTMLVFormFieldElement, FormField } from 'components/Form/FormField';

function normalizeIds(defaultValue: DocumentId | DocumentId[] | undefined | null): DocumentId[] {
  if (!defaultValue) {
    return [];
  }

  if (Array.isArray(defaultValue)) {
    return defaultValue;
  }

  return [defaultValue];
}

type Props = {
  id?: string;
  className?: string;
  documentTypes: DocumentType[];
  defaultValue?: DocumentId | DocumentId[];
  name: string;
  multiple?: boolean;
  readonly?: boolean;
  required?: boolean;
  disabled?: boolean;
  onChange?: (ids: DocumentId[]) => void;
};

export function RefInput({
  id,
  className,
  documentTypes,
  defaultValue: defaultValueRaw,
  name,
  multiple = false,
  readonly = false,
  required = false,
  disabled = false,
  onChange,
}: Props) {
  const fieldRef = useRef<HTMLVFormFieldElement>(null);

  const defaultValue = normalizeIds(defaultValueRaw);
  const [ids, setIds] = useState(defaultValue);

  const [showPicker, setShowPicker] = useState(false);

  const onChangeRef = useLatestRef(onChange);
  useEffect(() => {
    const el = fieldRef.current;
    if (!el) {
      throw new Error('v-form-field element is missing');
    }

    if (ids.length > 1 && !multiple) {
      setIds([ids[0]]);
      return;
    }

    if (multiple) {
      el.value = ids;
    } else {
      el.value = ids[0];
    }

    onChangeRef.current?.(ids);
  }, [onChangeRef, ids, multiple]);

  useEffect(() => {
    const el = fieldRef.current;
    if (!el) {
      throw new Error('v-form-field element is missing');
    }

    const form = el.form;
    if (!form) {
      throw new Error('form is missing');
    }

    const onReset = () => {
      setIds(el.value as DocumentId[]);
    };

    form.addEventListener('reset', onReset);

    return () => {
      form.removeEventListener('reset', onReset);
    };
  }, []);

  const { documents, error, inProgress } = useDocuments(ids);

  const canAdd = ids.length === 0 || multiple;

  return (
    <FormField
      innerRef={fieldRef}
      id={id}
      className={cx(
        'ref-input inline-block break-all border-none',
        multiple && 'is-multi',
        className
      )}
      name={name}
      defaultValue={defaultValue}
      disabled={disabled}
      required={required}
    >
      {showPicker && (
        <DocumentPicker
          documentTypes={documentTypes}
          onSelected={(documentId) => {
            if (!ids.includes(documentId)) {
              setIds([...ids, documentId]);
            }
            setShowPicker(false);
          }}
          onCancel={() => setShowPicker(false)}
        />
      )}

      {error && <QueryError error={error} />}

      {documents?.map((item) => (
        <div className="flex items-center gap-4" key={item.id}>
          <Ref
            documentId={item.id}
            documentType={item.documentType}
            subtype={item.subtype}
            documentTitle={item.title}
          />

          {!readonly && !disabled && (
            <IconButton
              icon="x"
              size="sm"
              onClick={() => setIds(ids.filter((id) => id !== item.id))}
            />
          )}
        </div>
      ))}

      {canAdd && (
        <Button
          variant="text"
          onClick={() => setShowPicker(true)}
          disabled={readonly || disabled}
          busy={inProgress}
        >
          Pick {documentTypes.join(', ')}...
        </Button>
      )}
    </FormField>
  );
}
