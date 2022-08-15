import { useQuery } from '../hooks';
import { RPC } from '../rpc';
import { QueryError } from './QueryError';

type MarkupProps = {
  markup: string;
  className?: string;
};

export function Markup({ markup, className = '' }: MarkupProps) {
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.RenderMarkup({ markup }, abortSignal),
    [markup]
  );

  if (error) {
    return <QueryError error={error} />;
  }

  if (inProgress || !result) {
    return null;
  }

  return (
    <div className={`markup ${className}`} dangerouslySetInnerHTML={{ __html: result.html }} />
  );
}
