import { useState } from 'preact/hooks';
import { DocumentType, DocumentData, DocumentId, DocumentSubtype } from 'dto';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { Callback, copyTextToClipbard, getDocumentUrl } from 'utils';
import { isAttachment, isErasedDocument } from 'utils/schema';
import { QueryError } from 'components/QueryError';
import { IconButton } from 'components/Button';
import { Icon } from 'components/Icon';
import { getAttachmentPreview } from 'components/Ref';
import { DropdownMenu } from 'components/DropdownMenu';
import { DocumentViewerFields } from './DocumentViewerFields';
import { DocumentViewerHead } from './DocumentViewerHead';
import { CardContainer } from '../CardContainer';
import { EraseDocumentConfirmationDialog } from './EraseDocumentConfirmationDialog';

type DocumentViewerProps = {
  documentId: DocumentId;
  onEdit: Callback;
  onClone: (documentType: DocumentType, subtype: DocumentSubtype, data: DocumentData) => void;
};

export function DocumentViewer({ documentId, onEdit, onClone }: DocumentViewerProps) {
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
                    text: `Clone ${result.documentType}`,
                    icon: 'duplicate-document',
                    onClick: () => {
                      onClone(result.documentType, result.subtype, result.data);
                    },
                  },
                  {
                    text: `Erase ${result.documentType}`,
                    icon: 'erase-document',
                    alarming: true,
                    onClick: () => setShowEraseDocumentConfirmationDialog(true),
                  },
                ]}
              />

              <IconButton
                icon="pencil-square"
                title={`Edit ${result.documentType}`}
                onClick={onEdit}
                size="lg"
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
            collections={result.collections}
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
            documentType={result.documentType}
            subtype={result.subtype}
            data={result.data}
          />

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
