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
import { isAttachment, isDocumentTypeCollection, isImageAttachment } from '../../schema';
import { DocumentData } from '../../dto';
import { CollectionCatalog } from './CollectionCatalog';

type DocumentViewerProps = {
  documentId: string;
  onEdit: Callback;
  query?: string;
  page?: number;
};

export function DocumentViewer({ documentId, onEdit, query, page }: DocumentViewerProps) {
  const { result, error, inProgress, triggerRefresh } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    {
      refreshIfChange: [documentId],
    }
  );

  return (
    <>
      <CardContainer.Topbar
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

                <Button variant="text" leadingIcon="edit-document" onClick={onEdit}>
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

          {isDocumentTypeCollection(result.documentType) && (
            <CollectionCatalog collectionId={documentId} query={query} page={page} />
          )}

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
