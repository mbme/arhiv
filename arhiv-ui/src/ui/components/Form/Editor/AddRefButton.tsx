import { useState } from 'react';
import { DocumentId, DocumentSubtype, DocumentType } from 'dto';
import { IconButton } from 'components/Button';
import { DocumentPicker } from 'components/DocumentPicker';

type Props = {
  className?: string;
  onDocumentSelected: (
    id: DocumentId,
    documentType: DocumentType,
    subtype: DocumentSubtype
  ) => void;
};

export function AddRefButton({ className, onDocumentSelected }: Props) {
  const [showPicker, setShowPicker] = useState(false);

  return (
    <>
      <IconButton icon="link" className={className} onClick={() => setShowPicker(true)} />

      {showPicker && (
        <DocumentPicker
          hideOnSelect
          onSelected={(documentId, documentType, subtype) => {
            setShowPicker(false);
            onDocumentSelected(documentId, documentType, subtype);
          }}
          onCancel={() => setShowPicker(false)}
        />
      )}
    </>
  );
}
