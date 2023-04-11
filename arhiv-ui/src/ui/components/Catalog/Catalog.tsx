import { useState } from 'react';
import { DocumentId, DocumentSubtype, DocumentType } from 'dto';
import { useUpdateEffect } from 'utils/hooks';
import { useSuspenseQuery } from 'utils/suspense';
import { DateTime } from 'components/DateTime';
import { SearchInput } from 'components/SearchInput';
import { Pagination } from './Pagination';

type CatalogProps = {
  autofocus?: boolean;
  className?: string;
  documentTypes?: DocumentType[];
  initialQuery?: string;
  initialPage?: number;
  onQueryChange?: (query: string) => void;
  onPageChange?: (page: number) => void;
  onDocumentSelected: (
    id: DocumentId,
    documentType: DocumentType,
    subtype: DocumentSubtype
  ) => void;
};

export function Catalog({
  autofocus = false,
  className,
  documentTypes = [],
  initialQuery = '',
  initialPage = 0,
  onQueryChange,
  onPageChange,
  onDocumentSelected,
}: CatalogProps) {
  const [query, _setQuery] = useState(initialQuery);
  const [page, _setPage] = useState(initialPage);

  const {
    value: result,
    isUpdating,
    triggerRefresh,
  } = useSuspenseQuery({ typeName: 'ListDocuments', query, page, documentTypes });

  useUpdateEffect(() => {
    triggerRefresh();
  }, [query, page, ...documentTypes]);

  const setQuery = (query: string) => {
    _setQuery(query);
    onQueryChange?.(query);
  };

  const setPage = (page: number) => {
    _setPage(page);
    onPageChange?.(page);
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
      <SearchInput
        initialValue={query}
        onSearch={(newQuery) => {
          setQuery(newQuery);
          setPage(0);
        }}
        busy={isUpdating}
        autofocus={autofocus}
        debounceMs={400}
      />

      <div className="divide-y mb-6">
        {items}
        {items.length === 0 && <div className="text-center">No results 😿</div>}
      </div>

      <Pagination page={page} hasMore={result.hasMore} onClick={(newPage) => setPage(newPage)} />
    </div>
  );
}
