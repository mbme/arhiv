import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { DocumentFields } from './DocumentFields';
import { QueryError } from '../QueryError';
import { getFieldDescriptions } from '../../schema';
import { DocumentHead } from './DocumentHead';

type DocumentViewerProps = {
  documentId: string;
};

export function DocumentViewer({ documentId }: DocumentViewerProps) {
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    []
  );

  return (
    <div className="p-8">
      <QueryError error={error} />

      {inProgress && <div className="mb-8">Loading...</div>}

      {result && (
        <>
          <DocumentHead
            id={result.id}
            documentType={result.documentType}
            subtype={result.subtype}
            updatedAt={result.updatedAt}
          />
          <DocumentFields
            data={result.data}
            fields={getFieldDescriptions(result.documentType, result.subtype)}
          />
        </>
      )}
    </div>
  );
}
