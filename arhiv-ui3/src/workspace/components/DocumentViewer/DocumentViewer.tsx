import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { DocumentFields } from './DocumentFields';
import { QueryError } from '../QueryError';

type DocumentViewerProps = {
  documentId: string;
};

export function DocumentViewer({ documentId }: DocumentViewerProps) {
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    []
  );

  return (
    <div>
      <QueryError error={error} />

      {inProgress && <div className="mb-8">Loading...</div>}

      {result && (
        <DocumentFields
          data={result.data}
          dataDescription={result.dataDescription}
          subtype={result.subtype}
        />
      )}
    </div>
  );
}
