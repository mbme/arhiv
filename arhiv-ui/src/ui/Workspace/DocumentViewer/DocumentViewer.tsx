import { useQuery } from '../../utils/hooks';
import { RPC } from '../../utils/rpc';
import { Callback, formatDocumentType } from '../../utils';
import { isAttachment, isDocumentTypeCollection } from '../../utils/schema';
import { QueryError } from '../../components/QueryError';
import { Button } from '../../components/Button';
import { Icon } from '../../components/Icon';
import { getAttachmentPreview } from '../../components/Ref';
import { DocumentViewerFields } from './DocumentViewerFields';
import { DocumentViewerHead } from './DocumentViewerHead';
import { CardContainer } from '../CardContainer';
import { DocumentViewerBackrefs } from './DocumentViewerBackrefs';
import { EraseDocumentButton } from './EraseDocumentButton';
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
