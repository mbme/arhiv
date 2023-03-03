import { useEffect, useRef, useState } from 'preact/hooks';
import { cx } from 'utils';
import { useLatestRef } from 'utils/hooks';
import { DocumentId, DocumentType } from 'dto';
import { Ref, useDocuments } from 'components/Ref';
import { DocumentPicker } from 'components/DocumentPicker';
import { Button, IconButton } from 'components/Button';
import { QueryError } from 'components/QueryError';
import { HTMLVFormFieldElement } from 'components/Form/v-form-field';

type Props = {
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
  className,
  documentTypes,
  defaultValue,
  name,
  multiple = false,
  readonly = false,
  required = false,
  disabled = false,
  onChange,
}: Props) {
  const fieldRef = useRef<HTMLVFormFieldElement>(null);

  const [ids, setIds] = useState(() => {
    if (!defaultValue) {
      return [];
    }
    if (Array.isArray(defaultValue)) {
      return defaultValue;
    }

    return [defaultValue];
  });

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

  const { documents, error, inProgress } = useDocuments(ids);

  const canAdd = ids.length === 0 || multiple;

  return (
    <v-form-field
      ref={fieldRef}
      className={cx(
        'ref-input inline-block break-all border-none',
        multiple && 'is-multi',
        className
      )}
      name={name}
      defaultValue={JSON.stringify(defaultValue)}
      disabled={disabled}
      readonly={readonly}
      required={required}
      onChange={(e) => setIds(e.value as DocumentId[])}
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
    </v-form-field>
  );
}
