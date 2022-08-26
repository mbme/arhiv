import { useQuery } from '../hooks';
import { RPC } from '../rpc';
import { CardContainer } from './CardContainer';
import { QueryError } from './QueryError';
import { Spinner } from './Spinner';

export function StatusCard() {
  const { result, error, inProgress } = useQuery((abortSignal) => RPC.GetStatus({}, abortSignal));

  return (
    <>
      <CardContainer.Topbar title="Status" right={<CardContainer.CloseButton />} />

      {error && <QueryError error={error} />}

      {inProgress && <Spinner className="mb-8" />}

      {result && (
        <pre className="text-sm">
          <code>{result.status}</code>
        </pre>
      )}
    </>
  );
}
