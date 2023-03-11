import { useState } from 'preact/hooks';
import { DocumentId, DocumentSubtype, DocumentType } from 'dto';
import { cx } from 'utils';
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
      <IconButton
        icon="link"
        className={cx('bg-indigo-100 drop-shadow-md', className)}
        onClick={() => setShowPicker(true)}
      />

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
