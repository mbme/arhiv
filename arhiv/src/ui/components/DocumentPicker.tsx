import { useRef, useState } from 'react';
import { DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Catalog, CatalogInfo, DocumentInfo } from 'components/Catalog/Catalog';

export { DocumentInfo };

type Props = {
  documentTypes?: DocumentType[];
  onSelected: (info: DocumentInfo) => void;
  onCancel: () => void;
  hideOnSelect?: boolean;
  title?: string;
  onCreateNote?: (title: string) => void;
  onConvertToCard?: (info: CatalogInfo) => void;
  initialQuery?: string;
};

export function DocumentPicker({
  documentTypes: initialDocumentTypes = [],
  onSelected,
  onCancel,
  hideOnSelect,
  title,
  onCreateNote,
  onConvertToCard,
  initialQuery = '',
}: Props) {
  const dialogRef = useRef<HTMLDivElement>(null);

  const [documentTypes, setDocumentTypes] = useState(initialDocumentTypes);
  const [page, setPage] = useState(0);
  const [query, setQuery] = useState(initialQuery);

  return (
    <Dialog
      innerRef={dialogRef}
      title={title || `Pick ${documentTypes.join(', ') || 'document'}`}
      onHide={onCancel}
      contentClassName="px-3 pb-0"
    >
      <Catalog
        autofocus
        filter={{ documentTypes }}
        query={query}
        page={page}
        onQueryChange={setQuery}
        onPageChange={setPage}
        onFilterChange={(filter) => {
          setDocumentTypes(filter.documentTypes);
        }}
        onDocumentSelected={(info) => {
          if (!dialogRef.current) {
            throw new Error('dialog element is missing');
          }

          if (hideOnSelect) {
            dialogRef.current.setAttribute('hidden', '');
          }

          onSelected(info);
        }}
        onCreateNote={onCreateNote}
        onConvertToCard={onConvertToCard}
      />
    </Dialog>
  );
}
