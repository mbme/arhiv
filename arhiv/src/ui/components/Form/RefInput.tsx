import { useRef, useState } from 'react';
import { cx, NominalType } from 'utils';
import { useSuspenseQuery } from 'utils/suspense';
import { ATTACHMENT_DOCUMENT_TYPE, DocumentId, DocumentType } from 'dto';
import { Ref as RefComponent } from 'components/Ref';
import { DocumentPicker } from 'components/DocumentPicker';
import { Link } from 'components/Link';
import { Button, IconButton } from 'components/Button';
import { HTMLVFormFieldElement, FormField } from 'components/Form/FormField';
import { AttachmentPreviewBlock, canPreview } from 'components/AttachmentPreview';
import { AttachmentUrlDialog } from 'components/AttachmentUrlDialog';

type AttachmentUrl = NominalType<string, 'AttachmentUrl'>;

export type Ref = DocumentId | AttachmentUrl;

const isAttachmentUrl = (value: Ref): value is AttachmentUrl =>
  value.startsWith('http://') || value.startsWith('https://');

const isDocumentId = (value: Ref): value is DocumentId => !isAttachmentUrl(value);

function normalizeRefs(
  defaultValue: Ref | Ref[] | undefined | null,
): Array<DocumentId | AttachmentUrl> {
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
  const [showUrlPicker, setShowUrlPicker] = useState(false);

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
  const canAddUrls = documentTypes.includes(ATTACHMENT_DOCUMENT_TYPE) && canAdd;

  return (
    <FormField
      innerRef={fieldRef}
      id={id}
      className={cx('ref-input', className, {
        'is-multi': multiple,
        'is-empty': refs.length === 0,
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
            <AttachmentPreviewBlock documentId={item.id} data={item.data} />
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

      {showUrlPicker && (
        <AttachmentUrlDialog
          onConfirm={(urlRaw) => {
            const url = urlRaw as AttachmentUrl;

            if (!refs.includes(url)) {
              updateRefs([...refs, url], true);
            }
            setShowUrlPicker(false);
          }}
          onCancel={() => {
            setShowUrlPicker(false);
          }}
        />
      )}

      {refs.filter(isAttachmentUrl).map((attachmentUrl) => (
        <div key={attachmentUrl}>
          <div className="flex items-center gap-4">
            <Link url={attachmentUrl}>{attachmentUrl}</Link>

            {!readonly && !disabled && (
              <IconButton
                icon="x"
                size="sm"
                onClick={() => {
                  updateRefs(
                    refs.filter((item) => item !== attachmentUrl),
                    true,
                  );
                }}
              />
            )}
          </div>
        </div>
      ))}

      {canAddUrls && (
        <Button
          variant="text"
          onClick={() => {
            setShowUrlPicker(true);
          }}
          disabled={readonly || disabled}
          busy={isUpdating}
          leadingIcon="link"
        >
          Add url
        </Button>
      )}
    </FormField>
  );
}
