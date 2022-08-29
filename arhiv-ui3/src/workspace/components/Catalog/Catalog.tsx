import { useState } from 'preact/hooks';
import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { CardContainer } from '../CardContainer';
import { DateTime } from '../DateTime';
import { QueryError } from '../QueryError';
import { Pagination } from './Pagination';
import { SearchInput } from './SearchInput';

type CatalogProps = {
  onDocumentSelected: (documentId: string) => void;
};

export function Catalog({ onDocumentSelected }: CatalogProps) {
  const [query, setQuery] = useState('');
  const [page, setPage] = useState(0);

  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.ListDocuments({ query, page }, abortSignal),
    {
      refreshIfChange: [query, page],
    }
  );

  const items = result?.documents.map((item) => (
    <div
      className="cursor-pointer px-2 py-3 transition-colors hover:bg-sky-100"
      key={item.id}
      onClick={() => onDocumentSelected(item.id)}
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter') {
          (e.target as HTMLElement).click();
        }
      }}
    >
      <div className="font-bold text-lg">
        [{item.documentType || 'erased'}] {item.title}
      </div>

      <DateTime className="font-mono text-sm" datetime={item.updatedAt} relative />
    </div>
  ));

  return (
    <>
      <CardContainer.Topbar
        left={<span className="section-heading text-lg">Catalog</span>}
        right={<CardContainer.CloseButton />}
      />

      <SearchInput
        value={query}
        onSearch={(newQuery) => {
          setQuery(newQuery);
          setPage(0);
        }}
        busy={inProgress}
      />

      {error && <QueryError error={error} />}

      <div className="divide-y mb-6">
        {items}
        {items?.length === 0 && <div className="text-center">No results :(</div>}
      </div>

      {result && (
        <Pagination page={page} hasMore={result.hasMore} onClick={(newPage) => setPage(newPage)} />
      )}
    </>
  );
}
