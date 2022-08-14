import { useState } from 'preact/hooks';
import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { DateTime } from '../DateTime';
import { QueryError } from '../QueryError';
import { SearchInput } from './SearchInput';

type CatalogProps = {
  hidden?: boolean;
  onDocumentSelected: (documentId: string) => void;
};

export function Catalog({ hidden, onDocumentSelected }: CatalogProps) {
  const [query, setQuery] = useState('');

  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.ListDocuments({ query }, abortSignal),
    [query]
  );

  const items = result?.documents.map((item) => (
    <div
      className="mb-4 cursor-pointer bg-zinc-100 px-4 py-2"
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
    <div className="p-8" hidden={hidden}>
      <SearchInput value={query} onSearch={setQuery} />

      <QueryError error={error} />

      {inProgress && <div className="mb-8">Loading...</div>}

      {items}

      {result?.hasMore && <h2>HAS MORE</h2>}
    </div>
  );
}
