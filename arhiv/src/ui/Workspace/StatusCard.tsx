import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Icon } from 'components/Icon';
import { QueryError } from 'components/QueryError';
import { CardContainer } from './CardContainer';

export function StatusCard() {
  const { result, error, inProgress } = useQuery((abortSignal) => RPC.GetStatus({}, abortSignal));

  return (
    <CardContainer title="ARHIV STATUS">
      {error && <QueryError error={error} />}

      {inProgress && <Icon variant="spinner" className="mb-8" />}

      {result && (
        <pre className="text-sm">
          <code>{result.status}</code>
        </pre>
      )}
    </CardContainer>
  );
}
