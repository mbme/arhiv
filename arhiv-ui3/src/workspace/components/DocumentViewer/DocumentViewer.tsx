import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { DocumentViewerFields } from './DocumentViewerFields';
import { QueryError } from '../QueryError';
import { DocumentViewerHead } from './DocumentViewerHead';
import { Callback } from '../../../scripts/utils';
import { Button } from '../Button';
import { CardContainer } from '../CardContainer';
import { DocumentViewerBackrefs } from './DocumentViewerBackrefs';
import { EraseDocumentButton } from './EraseDocumentButton';

type DocumentViewerProps = {
  documentId: string;
  onBack?: Callback;
  onEdit: Callback;
};

export function DocumentViewer({ documentId, onBack, onEdit }: DocumentViewerProps) {
  const { result, error, inProgress, triggerRefresh } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    {
      refreshIfChange: [documentId],
    }
  );

  return (
    <>
      <CardContainer.Topbar
        title="Viewer"
        left={
          onBack && (
            <Button variant="text" icon="arrow-left" onClick={onBack}>
              Back
            </Button>
          )
        }
        right={
          <>
            {result?.documentType && (
              <>
                <EraseDocumentButton
                  documentId={result.id}
                  documentType={result.documentType}
                  title={result.title}
                  onErase={triggerRefresh}
                />

                <Button variant="text" icon="edit-document" onClick={onEdit}>
                  Edit
                </Button>
              </>
            )}

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
