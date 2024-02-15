import { useEffect, useRef, useState } from 'react';
import { cx } from 'utils';
import { useLatestRef } from 'utils/hooks';
import { useSuspenseQuery } from 'utils/suspense';
import { DocumentId, DocumentType } from 'dto';
import { Ref } from 'components/Ref';
import { DocumentPicker } from 'components/DocumentPicker';
import { Button, IconButton } from 'components/Button';
import { HTMLVFormFieldElement, FormField } from 'components/Form/FormField';
import { AttachmentPreviewBlock, canPreview } from 'components/AttachmentPreview';

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

  const updateIds = (newIds: DocumentId[], triggerChange: boolean) => {
    if (newIds.length > 1 && !multiple) {
      return;
    }

    const el = fieldRef.current;
    if (!el) {
      throw new Error('v-form-field element is missing');
    }

    const value = multiple ? newIds : newIds[0];
    if (triggerChange) {
      el.inputValue(value);
    } else {
      el.value = value;
    }

    setIds(newIds);
  };

  const onChangeRef = useLatestRef(onChange);
  useEffect(() => {
    onChangeRef.current?.(ids);
  }, [onChangeRef, ids]);

  const { value, isUpdating } = useSuspenseQuery({
    typeName: 'GetDocuments',
    ids,
  });

  const canAdd = ids.length === 0 || multiple;

  return (
    <FormField
      innerRef={fieldRef}
      id={id}
      className={cx(
        'ref-input inline-block break-all border-none',
        multiple && 'is-multi',
        className,
      )}
      name={name}
      defaultValue={defaultValueRaw}
      disabled={disabled}
      required={required}
      onReset={() => {
        updateIds(defaultValue, false);
      }}
    >
      {showPicker && (
        <DocumentPicker
          documentTypes={documentTypes}
          onSelected={({ id }) => {
            if (!ids.includes(id)) {
              updateIds([...ids, id], true);
            }
            setShowPicker(false);
          }}
          onCancel={() => setShowPicker(false)}
        />
      )}

      {value?.documents.map((item) => (
        <div key={item.id}>
          <div className="flex items-center gap-4">
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
                onClick={() => {
                  updateIds(
                    ids.filter((id) => id !== item.id),
                    true,
                  );
                }}
              />
            )}
          </div>

          {canPreview(item.documentType, item.subtype) && (
            <AttachmentPreviewBlock documentId={item.id} subtype={item.subtype} data={item.data} />
          )}
        </div>
      ))}

      {canAdd && (
        <Button
          variant="text"
          onClick={() => setShowPicker(true)}
          disabled={readonly || disabled}
          busy={isUpdating}
        >
          Pick {documentTypes.join(', ')}...
        </Button>
      )}
    </FormField>
  );
}
