import { useState } from 'preact/hooks';
import { RefContainer } from 'components/Ref';
import { DocumentPicker } from 'components/DocumentPicker';
import { Button, IconButton } from 'components/Button';

type Props = {
  documentType: string;
  ids: string[];
  multiple: boolean;
  readonly: boolean;
  disabled: boolean;
  onChange: (ids: string[]) => void;
  onRefClick: (documentId: string) => void;
};

export function RefInput({
  documentType,
  ids,
  multiple,
  readonly,
  disabled,
  onChange,
  onRefClick,
}: Props) {
  const [showPicker, setShowPicker] = useState(false);

  const canAdd = ids.length === 0 || multiple;

  return (
    <>
      {showPicker && (
        <DocumentPicker
          documentType={documentType}
          onSelected={(documentId) => {
            onChange([...ids, documentId]);
            setShowPicker(false);
          }}
          onCancel={() => setShowPicker(false)}
        />
      )}
      {ids.map((documentId) => (
        <div className="flex items-center gap-4" key={documentId}>
          <RefContainer id={documentId} onClick={() => onRefClick(documentId)} />

          {!readonly && !disabled && (
            <IconButton
              icon="x"
              size="sm"
              onClick={() => onChange(ids.filter((id) => id !== documentId))}
            />
          )}
        </div>
      ))}

      {canAdd && (
        <Button variant="text" onClick={() => setShowPicker(true)} disabled={readonly || disabled}>
          Pick {documentType}...
        </Button>
      )}
    </>
  );
}
