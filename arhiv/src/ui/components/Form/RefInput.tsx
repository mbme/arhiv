import { useRef, useState } from 'react';
import { cx, NominalType } from 'utils';
import { useSuspenseQuery } from 'utils/suspense';
import { DocumentId, DocumentType } from 'dto';
import { Ref as RefComponent } from 'components/Ref';
import { DocumentPicker } from 'components/DocumentPicker';
import { Link } from 'components/Link';
import { Button, IconButton } from 'components/Button';
import { HTMLVFormFieldElement, FormField } from 'components/Form/FormField';
import { AssetPreviewBlock, canPreview } from 'components/AssetPreview';

type AssetUrl = NominalType<string, 'AssetUrl'>;

export type Ref = DocumentId | AssetUrl;

const isAssetUrl = (value: Ref): value is AssetUrl =>
  value.startsWith('http://') || value.startsWith('https://');

const isDocumentId = (value: Ref): value is DocumentId => !isAssetUrl(value);

function normalizeRefs(defaultValue: Ref | Ref[] | undefined | null): Array<DocumentId | AssetUrl> {
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
  defaultValue?: Ref | Ref[];
  name: string;
  multiple?: boolean;
  readonly?: boolean;
  required?: boolean;
  disabled?: boolean;
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
}: Props) {
  const fieldRef = useRef<HTMLVFormFieldElement<Ref | Ref[]>>(null);

  const defaultValue = normalizeRefs(defaultValueRaw);

  const [refs, setRefs] = useState(defaultValue);

  const [showDocumentPicker, setShowDocumentPicker] = useState(false);

  const updateRefs = (newRefs: Ref[], triggerChange: boolean) => {
    if (newRefs.length > 1 && !multiple) {
      return;
    }

    const el = fieldRef.current;
    if (!el) {
      throw new Error('v-form-field element is missing');
    }

    const value = multiple ? newRefs : (newRefs[0] ?? null);
    if (triggerChange) {
      el.inputValue(value);
    } else {
      el.value = value;
    }

    setRefs(newRefs);
  };

  const { value, isUpdating } = useSuspenseQuery({
    typeName: 'GetDocuments',
    ids: refs.filter(isDocumentId),
  });

  const canAdd = refs.length === 0 || multiple;

  return (
    <FormField
      innerRef={fieldRef}
      id={id}
      className={cx('inline-block break-all border-none', className, {
        'w-full': multiple,
        'inline-flex justify-around': refs.length === 0,
      })}
      name={name}
      defaultValue={defaultValueRaw}
      disabled={disabled}
      required={required}
      onReset={() => {
        updateRefs(defaultValue, false);
      }}
    >
      {showDocumentPicker && (
        <DocumentPicker
          documentTypes={documentTypes}
          onSelected={({ id }) => {
            if (!refs.includes(id)) {
              updateRefs([...refs, id], true);
            }
            setShowDocumentPicker(false);
          }}
          onCancel={() => {
            setShowDocumentPicker(false);
          }}
        />
      )}

      {value.documents.map((item) => (
        <div key={item.id}>
          <div className="flex items-center gap-4">
            <RefComponent
              documentId={item.id}
              documentType={item.documentType}
              documentTitle={item.title}
            />

            {!readonly && !disabled && (
              <IconButton
                icon="x"
                size="sm"
                onClick={() => {
                  updateRefs(
                    refs.filter((ref) => ref !== item.id),
                    true,
                  );
                }}
              />
            )}
          </div>

          {canPreview(item.documentType, item.data) && (
            <AssetPreviewBlock documentId={item.id} data={item.data} />
          )}
        </div>
      ))}

      {canAdd && (
        <Button
          variant="text"
          onClick={() => {
            setShowDocumentPicker(true);
          }}
          disabled={readonly || disabled}
          busy={isUpdating}
          leadingIcon="eye"
        >
          Pick {documentTypes.join(', ')}
        </Button>
      )}

      {refs.filter(isAssetUrl).map((assetUrl) => (
        <div key={assetUrl}>
          <div className="flex items-center gap-4">
            <Link url={assetUrl}>{assetUrl}</Link>

            {!readonly && !disabled && (
              <IconButton
                icon="x"
                size="sm"
                onClick={() => {
                  updateRefs(
                    refs.filter((item) => item !== assetUrl),
                    true,
                  );
                }}
              />
            )}
          </div>
        </div>
      ))}
    </FormField>
  );
}
