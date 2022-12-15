import { useState } from 'preact/hooks';
import { RefContainer } from 'components/Ref';
import { DocumentPicker } from 'components/DocumentPicker';
import { Button, IconButton } from 'components/Button';

type Props = {
  documentType: string;
  documentId?: string;
  readonly: boolean;
  disabled: boolean;
  onChange: (documentId?: string) => void;
  onRefClick: (documentId: string) => void;
};

export function RefInput({
  documentType,
  documentId,
  readonly,
  disabled,
  onChange,
  onRefClick,
}: Props) {
  const [showPicker, setShowPicker] = useState(false);

  if (showPicker) {
    return (
      <DocumentPicker
        documentType={documentType}
        onSelected={(documentId) => {
          onChange(documentId);
          setShowPicker(false);
        }}
        onCancel={() => setShowPicker(false)}
      />
    );
  }

  if (documentId) {
    return (
      <div className="flex items-center gap-4">
        <RefContainer id={documentId} onClick={() => onRefClick(documentId)} />

        {!readonly && !disabled && (
          <IconButton icon="x" size="sm" onClick={() => onChange(undefined)} />
        )}
      </div>
    );
  }

  return (
    <Button variant="text" onClick={() => setShowPicker(true)} disabled={readonly || disabled}>
      Pick {documentType}...
    </Button>
  );
}
