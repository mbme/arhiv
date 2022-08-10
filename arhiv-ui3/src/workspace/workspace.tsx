import { render } from 'preact';
import { useEffect, useState } from 'preact/hooks';
import { formatDate, formatDateHuman } from '../scripts/date';
import { RPC } from './rpc';

const renderRoot = document.querySelector('main');
if (!renderRoot) {
  throw new Error('render root not found');
}

type SearchInputProps = {
  value: string;
  onSearch: (query: string) => void;
};
function SearchInput({ value, onSearch }: SearchInputProps) {
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSearch((e.target as HTMLFormElement).querySelector('input')!.value);
      }}
    >
      <input
        type="search"
        name="pattern"
        class="field w-full mb-8"
        value={value}
        placeholder="Type something"
        autofocus
      />
    </form>
  );
}

function useQuery<TResult>(
  cb: (signal: AbortSignal) => Promise<TResult>,
  inputs: readonly unknown[]
): { result?: TResult; inProgress: boolean; error?: unknown } {
  const [inProgress, setInProgress] = useState(false);
  const [result, setResult] = useState<TResult>();
  const [error, setError] = useState<unknown>();

  useEffect(() => {
    const controller = new AbortController();
    setInProgress(true);

    cb(controller.signal).then(
      (result) => {
        setResult(result);
        setError(undefined);
        setInProgress(false);
      },
      (error) => {
        setResult(undefined);
        setError(error);
        setInProgress(false);
      }
    );

    return () => {
      controller.abort();
      setInProgress(false);
    };
  }, inputs);

  return {
    result,
    error,
    inProgress,
  };
}

type RelTimeProps = {
  datetime: string;
  className?: string;
};

function RelTime({ datetime, className }: RelTimeProps) {
  const date = new Date(datetime);

  return (
    <time dateTime={datetime} title={formatDate(date)} className={className}>
      {formatDateHuman(date)}
    </time>
  );
}

function Workspace() {
  const [query, setQuery] = useState('');

  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.ListDocuments({ query }, abortSignal),
    [query]
  );

  const items = result?.documents.map((item) => (
    <div className="mb-4 cursor-pointer bg-zinc-100 px-4 py-2" key={item.id}>
      <div className="font-bold text-lg">
        [{item.documentType || 'erased'}] {item.title}
      </div>

      <RelTime className="font-mono text-sm" datetime={item.updatedAt} />
    </div>
  ));

  return (
    <div className="p-8">
      <SearchInput value={query} onSearch={setQuery} />

      {inProgress && <div className="mb-8">Loading...</div>}

      {error && (
        <pre>
          <code>{JSON.stringify(error)}</code>
        </pre>
      )}

      {items}

      {result?.hasMore && <h2>HAS MORE</h2>}
    </div>
  );
}

render(<Workspace />, renderRoot);
