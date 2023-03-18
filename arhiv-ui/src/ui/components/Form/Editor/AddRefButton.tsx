import { useState } from 'react';
import { IconButton } from 'components/Button';
import { DocumentInfo, DocumentPicker } from 'components/DocumentPicker';

type Props = {
  className?: string;
  onDocumentSelected: (info: DocumentInfo) => void;
};

export function AddRefButton({ className, onDocumentSelected }: Props) {
  const [showPicker, setShowPicker] = useState(false);

  if (showPicker) {
    return (
      <DocumentPicker
        hideOnSelect
        onSelected={(info) => {
          setShowPicker(false);
          onDocumentSelected(info);
        }}
        onCancel={() => setShowPicker(false)}
      />
    );
  }

  return <IconButton icon="link" className={className} onClick={() => setShowPicker(true)} />;
}
