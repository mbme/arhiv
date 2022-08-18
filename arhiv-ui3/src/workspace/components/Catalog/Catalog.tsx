import { useState } from 'preact/hooks';
import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { DateTime } from '../DateTime';
import { QueryError } from '../QueryError';
import { SearchInput } from './SearchInput';

type CatalogProps = {
  onDocumentSelected: (documentId: string) => void;
};

export function Catalog({ onDocumentSelected }: CatalogProps) {
  const [query, setQuery] = useState('');

  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.ListDocuments({ query }, abortSignal),
    [query]
  );

  const items = result?.documents.map((item) => (
    <div
      className="cursor-pointer px-2 py-4 transition-colors hover:bg-sky-100"
      key={item.id}
      onClick={() => onDocumentSelected(item.id)}
    >
      <div className="font-bold text-lg">
        [{item.documentType || 'erased'}] {item.title}
      </div>

      <DateTime className="font-mono text-sm" datetime={item.updatedAt} relative />
    </div>
  ));

  return (
    <div>
      <SearchInput value={query} onSearch={setQuery} />

      {error && <QueryError error={error} />}

      {inProgress && <div className="mb-8">Loading...</div>}

      <div className="divide-y">{items}</div>

      {result?.hasMore && <h2>HAS MORE</h2>}
    </div>
  );
}
