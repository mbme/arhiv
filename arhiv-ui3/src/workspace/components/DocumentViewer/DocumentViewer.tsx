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
import { Icon } from '../Icon';
import { isAttachment, isImageAttachment } from '../../schema';
import { DocumentData } from '../../dto';

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

      {inProgress && <Icon variant="spinner" className="mb-8" />}

      {result && (
        <>
          <DocumentViewerHead
            id={result.id}
            documentType={result.documentType}
            subtype={result.subtype}
            updatedAt={result.updatedAt}
          />

          {isAttachment(result.documentType) && (
            <div className="mb-8 empty:hidden">
              {getAttachmentPreview(result.subtype, result.data)}
            </div>
          )}

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

export function getAttachmentPreview(subtype: string, data: DocumentData) {
  const filename = data['filename'] as string;
  const blobId = data['blob'] as string;

  const blobUrl = `/blobs/${blobId}`;

  if (isImageAttachment(subtype)) {
    return <img src={blobUrl} alt={filename} className="max-h-96 mx-auto" />;
  }

  return null;
}
