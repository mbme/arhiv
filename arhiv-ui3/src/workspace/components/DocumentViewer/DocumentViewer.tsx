import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { DocumentViewerFields } from './DocumentViewerFields';
import { QueryError } from '../QueryError';
import { DocumentViewerHead } from './DocumentViewerHead';
import { Callback } from '../../../scripts/utils';

type DocumentViewerProps = {
  documentId: string;
  onClose: Callback;
  onEdit: Callback;
};

export function DocumentViewer({ documentId, onClose, onEdit }: DocumentViewerProps) {
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    []
  );

  return (
    <div className="p-8">
      <div className="flex gap-2 justify-between bg-neutral-200 py-2 mb-12 sticky top-0 z-10">
        <button className="font-mono" onClick={onClose}>
          CLOSE
        </button>

        <button className="font-mono" onClick={onEdit}>
          EDIT
        </button>
      </div>

      <QueryError error={error} />

      {inProgress && <div className="mb-8">Loading...</div>}

      {result && (
        <>
          <DocumentViewerHead
            id={result.id}
            documentType={result.documentType}
            subtype={result.subtype}
            updatedAt={result.updatedAt}
          />
          <DocumentViewerFields
            documentType={result.documentType}
            subtype={result.subtype}
            data={result.data}
          />
        </>
      )}
    </div>
  );
}
