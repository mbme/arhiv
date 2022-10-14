import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { DocumentViewerFields } from './DocumentViewerFields';
import { QueryError } from '../QueryError';
import { DocumentViewerHead } from './DocumentViewerHead';
import { Callback, formatDocumentType } from '../../../scripts/utils';
import { Button } from '../Button';
import { CardContainer } from '../CardContainer';
import { DocumentViewerBackrefs } from './DocumentViewerBackrefs';
import { EraseDocumentButton } from './EraseDocumentButton';
import { Icon } from '../Icon';
import {
  isAttachment,
  isAudioAttachment,
  isDocumentTypeCollection,
  isImageAttachment,
} from '../../schema';
import { DocumentData } from '../../dto';
import { CollectionCatalog } from './CollectionCatalog';
import { AudioPlayer } from '../AudioPlayer/AudioPlayer';

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
        left={
          <span className="section-heading text-lg">
            {result ? formatDocumentType(result.documentType, result.subtype) : ''}
          </span>
        }
        right={
          <>
            {result?.documentType && (
              <Button variant="text" leadingIcon="edit-document" onClick={onEdit}>
                Edit
              </Button>
            )}

            <CardContainer.CloseButton />
          </>
        }
      />

      {error && <QueryError error={error} />}

      {inProgress && <Icon variant="spinner" className="mb-8" />}

      {result && (
        <>
          <DocumentViewerHead id={result.id} updatedAt={result.updatedAt} />

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

          {result.documentType && (
            <div className="flex justify-end mt-8">
              <EraseDocumentButton
                documentId={result.id}
                documentType={result.documentType}
                title={result.title}
                onErase={triggerRefresh}
              />
            </div>
          )}
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

  if (isAudioAttachment(subtype)) {
    return <AudioPlayer url={blobUrl} title="" artist="" />;
  }

  return null;
}
