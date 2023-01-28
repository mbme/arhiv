import { useState } from 'preact/hooks';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { DateTime } from 'components/DateTime';
import { QueryError } from 'components/QueryError';
import { Pagination } from './Pagination';
import { SearchInput } from './SearchInput';

type CatalogProps = {
  autofocus?: boolean;
  documentTypes?: string[];
  initialQuery?: string;
  initialPage?: number;
  onQueryChange?: (query: string) => void;
  onPageChange?: (page: number) => void;
  onDocumentSelected: (documentId: string) => void;
};

export function Catalog({
  autofocus = false,
  documentTypes = [],
  initialQuery = '',
  initialPage = 0,
  onQueryChange,
  onPageChange,
  onDocumentSelected,
}: CatalogProps) {
  const [query, _setQuery] = useState(initialQuery);
  const [page, _setPage] = useState(initialPage);

  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.ListDocuments({ query, page, documentTypes }, abortSignal),
    {
      refreshIfChange: [query, page, ...documentTypes],
    }
  );

  const setQuery = (query: string) => {
    _setQuery(query);
    onQueryChange?.(query);
  };

  const setPage = (page: number) => {
    _setPage(page);
    onPageChange?.(page);
  };

  const items = result?.documents.map((item) => (
    <div
      className="cursor-pointer px-2 py-3 transition-colors hover:bg-sky-100"
      key={item.id}
      onClick={() => onDocumentSelected(item.id)}
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
    <>
      <SearchInput
        initialValue={query}
        onSearch={(newQuery) => {
          setQuery(newQuery);
          setPage(0);
        }}
        busy={inProgress}
        autofocus={autofocus}
      />

      {error && <QueryError error={error} />}

      <div className="divide-y mb-6">
        {items}
        {items?.length === 0 && <div className="text-center">No results 😿</div>}
      </div>

      {result && (
        <Pagination page={page} hasMore={result.hasMore} onClick={(newPage) => setPage(newPage)} />
      )}
    </>
  );
}
