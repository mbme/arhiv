import { useQuery } from '../hooks';
import { RPC } from '../rpc';
import { QueryError } from './QueryError';

type RefProps = {
  id: string;
};

export function Ref({ id }: RefProps) {
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.GetRef({ id }, abortSignal),
    [id]
  );

  if (error) {
    return <QueryError error={error} />;
  }

  if (inProgress || !result) {
    return null;
  }

  const url = `/documents/${result.id}`; // FIXME remove this

  return (
    <a href={url} class="bg-yellow-300 bg-opacity-30 px-2 py-1 rounded-sm">
      <span class="font-mono uppercase text-gray-400 mr-4">
        {result.documentType}
        {result.subtype && <> / {result.subtype}</>}
      </span>
      {result.title}
    </a>
  );
}
