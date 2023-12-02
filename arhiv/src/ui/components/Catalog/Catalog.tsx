import { useState } from 'react';
import { DocumentId, DocumentSubtype, DocumentType } from 'dto';
import { useToggle } from 'utils/hooks';
import { useSuspenseQuery } from 'utils/suspense';
import { CatalogCardProps } from 'Workspace/workspace-reducer';
import { DateTime } from 'components/DateTime';
import { SearchInput } from 'components/SearchInput';
import { IconButton } from 'components/Button';
import { Pagination } from './Pagination';
import { DEFAULT_DOCUMENT_TYPES, DocumentTypeSettings } from './DocumentTypeSettings';

type CatalogProps = {
  autofocus?: boolean;
  className?: string;
  documentTypes?: DocumentType[];
  initialQuery?: string;
  initialPage?: number;
  onPropChange?: (props: Partial<CatalogCardProps>) => void;
  onDocumentSelected: (
    id: DocumentId,
    documentType: DocumentType,
    subtype: DocumentSubtype,
  ) => void;
};

export function Catalog({
  autofocus = false,
  className,
  documentTypes: initialDocumentTypes,
  initialQuery = '',
  initialPage = 0,
  onPropChange,
  onDocumentSelected,
}: CatalogProps) {
  const [query, _setQuery] = useState(initialQuery);
  const [page, _setPage] = useState(initialPage);
  const [showSettings, toggleSettings] = useToggle(false);
  const [documentTypes, setDocumentTypes] = useState(
    initialDocumentTypes ?? DEFAULT_DOCUMENT_TYPES,
  );

  const { value: result, isUpdating } = useSuspenseQuery({
    typeName: 'ListDocuments',
    query,
    page,
    documentTypes,
  });

  const setQuery = (query: string) => {
    _setQuery(query);
    onPropChange?.({ query });
  };

  const setPage = (page: number) => {
    _setPage(page);
    onPropChange?.({ page });
  };

  const items = result.documents.map((item) => (
    <div
      className="cursor-pointer px-2 py-3 transition-colors hover:bg-sky-100"
      key={item.id}
      onClick={() => onDocumentSelected(item.id, item.documentType, item.subtype)}
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter') {
          e.currentTarget.click();
        }
      }}
    >
      <div className="font-bold text-lg break-all">
        [{item.documentType || 'erased'}] {item.title}
      </div>

      <DateTime className="font-mono text-sm" datetime={item.updatedAt} relative />
    </div>
  ));

  return (
    <div className={className}>
      <div className="flex gap-4 items-center mb-4">
        <SearchInput
          className="flex-auto"
          initialValue={query}
          onSearch={(newQuery) => {
            setQuery(newQuery);
            setPage(0);
          }}
          busy={isUpdating}
          autofocus={autofocus}
          debounceMs={400}
        />

        <IconButton icon="cog" size="sm" onClick={toggleSettings} />
      </div>

      {showSettings && (
        <DocumentTypeSettings
          className="mb-4 px-2 py-2 bg-zinc-50"
          selectableTypes={initialDocumentTypes}
          selected={documentTypes}
          onChange={setDocumentTypes}
        />
      )}

      <div className="divide-y mb-6">
        {items}
        {items.length === 0 && <div className="text-center">No results 😿</div>}
      </div>

      <Pagination page={page} hasMore={result.hasMore} onClick={(newPage) => setPage(newPage)} />
    </div>
  );
}
