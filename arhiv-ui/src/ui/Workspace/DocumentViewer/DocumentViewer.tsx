import { useState } from 'preact/hooks';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { Callback, copyTextToClipbard, getDocumentUrl } from 'utils';
import { isAttachment, isDocumentTypeCollection, isErasedDocument } from 'utils/schema';
import { QueryError } from 'components/QueryError';
import { Button } from 'components/Button';
import { Icon } from 'components/Icon';
import { getAttachmentPreview } from 'components/Ref';
import { DropdownMenu } from 'components/DropdownMenu';
import { DocumentViewerFields } from './DocumentViewerFields';
import { DocumentViewerHead } from './DocumentViewerHead';
import { CardContainer } from '../CardContainer';
import { EraseDocumentConfirmationDialog } from './EraseDocumentConfirmationDialog';
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

  const [showEraseDocumentConfirmationDialog, setShowEraseDocumentConfirmationDialog] =
    useState(false);

  return (
    <>
      <CardContainer.Topbar
        left={
          result?.documentType ? (
            <>
              <Button variant="text" leadingIcon="edit-document" onClick={onEdit}>
                Edit
              </Button>

              <DropdownMenu
                icon="dots-horizontal"
                options={[
                  {
                    text: 'Copy link',
                    icon: 'clipboard',
                    onClick: () => {
                      void copyTextToClipbard(getDocumentUrl(result.id));
                    },
                  },
                  {
                    text: 'Erase document',
                    icon: 'erase-document',
                    alarming: true,
                    onClick: () => setShowEraseDocumentConfirmationDialog(true),
                  },
                ]}
              />
            </>
          ) : null
        }
        right={<CardContainer.CloseButton />}
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
            backrefs={result.backrefs}
          />

          {isAttachment(result.documentType) && (
            <div className="mb-8 empty:hidden">
              {getAttachmentPreview(result.subtype, result.data)}
            </div>
          )}

          {isErasedDocument(result.documentType) && (
            <img
              src="/public/nothing-to-see-here.jpg"
              alt="funny picture for the erased document"
              className="my-16 mx-auto"
            />
          )}

          <DocumentViewerFields
            id={result.id}
            updatedAt={result.updatedAt}
            documentType={result.documentType}
            subtype={result.subtype}
            data={result.data}
          />

          {isDocumentTypeCollection(result.documentType) && (
            <CollectionCatalog collectionId={documentId} query={query} page={page} />
          )}

          {showEraseDocumentConfirmationDialog && (
            <EraseDocumentConfirmationDialog
              documentId={result.id}
              documentType={result.documentType}
              title={result.title}
              onErase={triggerRefresh}
              onCancel={() => setShowEraseDocumentConfirmationDialog(false)}
            />
          )}
        </>
      )}
    </>
  );
}
