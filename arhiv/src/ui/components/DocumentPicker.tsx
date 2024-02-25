import { useRef, useState } from 'react';
import { DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Catalog, DocumentInfo } from 'components/Catalog/Catalog';

export { DocumentInfo };

type Props = {
  documentTypes?: DocumentType[];
  onSelected: (info: DocumentInfo) => void;
  onCancel: () => void;
  hideOnSelect?: boolean;
  title?: string;
  onCreateNote?: (title: string) => void;
};

export function DocumentPicker({
  documentTypes: initialDocumentTypes = [],
  onSelected,
  onCancel,
  hideOnSelect,
  title,
  onCreateNote,
}: Props) {
  const dialogRef = useRef<HTMLDivElement>(null);

  const [documentTypes, setDocumentTypes] = useState(initialDocumentTypes);
  const [page, setPage] = useState(0);
  const [query, setQuery] = useState('');
  const [showSettings, setShowSettings] = useState(false);

  return (
    <Dialog
      innerRef={dialogRef}
      title={title || `Pick ${documentTypes?.join(', ') ?? 'document'}`}
      onHide={onCancel}
      contentClassName="px-3 pb-0"
    >
      <Catalog
        autofocus
        documentTypes={documentTypes ?? []}
        query={query}
        page={page}
        showSettings={showSettings}
        onQueryChange={setQuery}
        onPageChange={setPage}
        onToggleSettings={setShowSettings}
        onIncludedDocumentTypesChange={setDocumentTypes}
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
      />
    </Dialog>
  );
}
