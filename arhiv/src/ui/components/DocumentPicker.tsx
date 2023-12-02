import { useRef, useState } from 'react';
import { DocumentId, DocumentSubtype, DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Catalog } from 'components/Catalog/Catalog';
import { noop } from 'utils';

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

  const [page, setPage] = useState(0);
  const [query, setQuery] = useState('');
  const [showSettings, setShowSettings] = useState(false);

  return (
    <Dialog
      innerRef={dialogRef}
      title={`Pick ${documentTypes?.join(', ') ?? 'document'}`}
      onHide={onCancel}
    >
      <Catalog
        className="px-2"
        autofocus
        documentTypes={documentTypes ?? []}
        query={query}
        page={page}
        showSettings={showSettings}
        onQueryChange={setQuery}
        onPageChange={setPage}
        onToggleSettings={setShowSettings}
        onIncludedDocumentTypesChange={noop}
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
    </Dialog>
  );
}
