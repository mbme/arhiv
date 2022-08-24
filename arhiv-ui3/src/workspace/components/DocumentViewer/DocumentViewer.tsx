import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { DocumentViewerFields } from './DocumentViewerFields';
import { QueryError } from '../QueryError';
import { DocumentViewerHead } from './DocumentViewerHead';
import { Callback } from '../../../scripts/utils';
import { Icon } from '../Icon';
import { Button } from '../Button';
import { CardContainer } from '../CardContainer';
import { DocumentViewerBackrefs } from './DocumentViewerBackrefs';

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
      <CardContainer.Topbar
        title="Viewer"
        left={
          onBack && (
            <Button variant="link" onClick={onBack}>
              <Icon variant="arrow-left" className="mr-2" />
              Back
            </Button>
          )
        }
        right={
          <>
            <Button variant="link" onClick={onEdit}>
              <Icon variant="document-edit" className="mr-2" />
              Edit
            </Button>

            <CardContainer.CloseButton />
          </>
        }
      />

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
          <DocumentViewerBackrefs backrefs={result.backrefs} />
        </>
      )}
    </>
  );
}
