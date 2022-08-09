import { render } from 'preact';
import { useEffect, useState } from 'preact/hooks';
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

function useQuery<Result>(
  cb: (signal: AbortSignal) => Promise<Result>,
  inputs: readonly unknown[]
): { result?: Result; inProgress: boolean; error?: unknown } {
  const [inProgress, setInProgress] = useState(false);
  const [result, setResult] = useState<Result>();
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

function Workspace() {
  const [query, setQuery] = useState('');

  const { result, error, inProgress } = useQuery(
    async (abortSignal) => {
      const { documents } = await RPC.ListDocuments({ query }, abortSignal);

      return documents;
    },
    [query]
  );

  return (
    <div>
      <SearchInput value={query} onSearch={setQuery} />

      {inProgress && <div className="mb-8">Loading...</div>}

      <pre>
        <code>{error ? JSON.stringify(error) : result}</code>
      </pre>
    </div>
  );
}

render(<Workspace />, renderRoot);
