import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { DocumentViewerFields } from './DocumentViewerFields';
import { QueryError } from '../QueryError';
import { DocumentViewerHead } from './DocumentViewerHead';
import { Callback } from '../../../scripts/utils';
import { Icon } from '../Icon';
import { Button } from '../Button';
import { CardContainer } from '../CardContainer';

type DocumentViewerProps = {
  documentId: string;
  onBack?: Callback;
  onEdit: Callback;
};

export function DocumentViewer({ documentId, onBack, onEdit }: DocumentViewerProps) {
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    [documentId]
  );

  return (
    <>
      <CardContainer.Topbar>
        {onBack && (
          <Button variant="simple" onClick={onBack} className="mr-auto">
            <Icon variant="arrow-left" className="mr-2" />
            Back
          </Button>
        )}

        <div className="flex gap-1">
          <Button variant="simple" onClick={onEdit}>
            <Icon variant="document-edit" className="mr-2" />
            Edit
          </Button>

          <CardContainer.CloseButton />
        </div>
      </CardContainer.Topbar>

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
    </>
  );
}
