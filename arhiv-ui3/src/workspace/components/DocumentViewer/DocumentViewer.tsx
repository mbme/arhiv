import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { DocumentViewerFields } from './DocumentViewerFields';
import { QueryError } from '../QueryError';
import { DocumentViewerHead } from './DocumentViewerHead';
import { Callback } from '../../../scripts/utils';
import { Icon } from '../Icon';
import { Button } from '../Button';

type DocumentViewerProps = {
  documentId: string;
  onClose: Callback;
  onEdit: Callback;
};

export function DocumentViewer({ documentId, onClose, onEdit }: DocumentViewerProps) {
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    [documentId]
  );

  return (
    <div>
      <div className="flex gap-2 justify-between bg-neutral-200 py-2 mb-12 sticky top-0 z-10">
        <Button variant="simple" onClick={onClose}>
          <Icon variant="arrow-left" className="mr-2" />
          Back
        </Button>

        <Button variant="simple" onClick={onEdit}>
          <Icon variant="document-edit" className="mr-2" />
          Edit
        </Button>
      </div>

      {error && <QueryError error={error} />}

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
