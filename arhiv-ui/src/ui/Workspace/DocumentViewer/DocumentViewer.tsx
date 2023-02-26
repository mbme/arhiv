import { Suspense } from 'preact/compat';
import { useState } from 'preact/hooks';
import { DocumentDTO } from 'dto';
import { Callback, copyTextToClipbard, getDocumentUrl } from 'utils';
import { isAttachment, isErasedDocument } from 'utils/schema';
import { Icon } from 'components/Icon';
import { IconButton } from 'components/Button';
import { getAttachmentPreview } from 'components/Ref';
import { DropdownMenu } from 'components/DropdownMenu';
import { DocumentViewerFields } from './DocumentViewerFields';
import { DocumentViewerHead } from './DocumentViewerHead';
import { CardContainer } from '../CardContainer';
import { EraseDocumentConfirmationDialog } from './EraseDocumentConfirmationDialog';

type DocumentViewerProps = {
  document: DocumentDTO;
  onEdit: Callback;
  onClone: Callback;
  onErase: Callback;
};

export function DocumentViewer({ document, onEdit, onClone, onErase }: DocumentViewerProps) {
  const [showEraseDocumentConfirmationDialog, setShowEraseDocumentConfirmationDialog] =
    useState(false);

  return (
    <CardContainer>
      <CardContainer.Topbar
        left={
          <>
            <DropdownMenu
              icon="dots-horizontal"
              options={[
                {
                  text: 'Copy link',
                  icon: 'clipboard',
                  onClick: () => {
                    void copyTextToClipbard(getDocumentUrl(document.id));
                  },
                },
                {
                  text: `Clone ${document.documentType}`,
                  icon: 'duplicate-document',
                  onClick: onClone,
                },
                {
                  text: `Erase ${document.documentType}`,
                  icon: 'erase-document',
                  alarming: true,
                  onClick: () => setShowEraseDocumentConfirmationDialog(true),
                },
              ]}
            />

            <IconButton
              icon="pencil-square"
              title={`Edit ${document.documentType}`}
              onClick={onEdit}
              size="lg"
            />
          </>
        }
        right={<CardContainer.CloseButton />}
      />

      <DocumentViewerHead
        id={document.id}
        documentType={document.documentType}
        subtype={document.subtype}
        updatedAt={document.updatedAt}
        backrefs={document.backrefs}
        collections={document.collections}
      />

      {isAttachment(document.documentType) && (
        <div className="mb-8 empty:hidden">
          {getAttachmentPreview(document.subtype, document.data)}
        </div>
      )}

      {isErasedDocument(document.documentType) && (
        <img
          src="/public/nothing-to-see-here.jpg"
          alt="funny picture for the erased document"
          className="my-16 mx-auto"
        />
      )}

      <Suspense fallback={<Icon variant="spinner" className="mb-8" />}>
        <DocumentViewerFields
          documentType={document.documentType}
          subtype={document.subtype}
          data={document.data}
        />
      </Suspense>

      {showEraseDocumentConfirmationDialog && (
        <EraseDocumentConfirmationDialog
          documentId={document.id}
          documentType={document.documentType}
          title={document.title}
          onErase={onErase}
          onCancel={() => setShowEraseDocumentConfirmationDialog(false)}
        />
      )}
    </CardContainer>
  );
}
