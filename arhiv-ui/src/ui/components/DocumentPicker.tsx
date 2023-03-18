import { useRef } from 'react';
import { DocumentId, DocumentSubtype, DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Catalog } from 'components/Catalog/Catalog';

export type DocumentInfo = {
  id: DocumentId;
  documentType: DocumentType;
  subtype: DocumentSubtype;
};

type Props = {
  documentTypes?: DocumentType[];
  onSelected: (info: DocumentInfo) => void;
  onCancel: () => void;
  hideOnSelect?: boolean;
};

export function DocumentPicker({ documentTypes, onSelected, onCancel, hideOnSelect }: Props) {
  const dialogRef = useRef<HTMLDivElement>(null);

  return (
    <Dialog
      innerRef={dialogRef}
      title={`Pick ${documentTypes?.join(', ') ?? 'document'}`}
      onHide={onCancel}
    >
      <div className="px-2">
        <Catalog
          autofocus
          documentTypes={documentTypes}
          onDocumentSelected={(id, documentType, subtype) => {
            if (!dialogRef.current) {
              throw new Error('dialog element is missing');
            }

            if (hideOnSelect) {
              dialogRef.current.setAttribute('hidden', '');
            }

            onSelected({
              id,
              documentType,
              subtype,
            });
          }}
        />
      </div>
    </Dialog>
  );
}
