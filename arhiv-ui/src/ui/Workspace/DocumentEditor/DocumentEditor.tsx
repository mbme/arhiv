import { useRef, useState } from 'react';
import { DocumentDTO } from 'dto';
import { Callback, copyTextToClipbard, getDocumentUrl } from 'utils';
import { RPC } from 'utils/rpc';
import { isAttachment, isErasedDocument } from 'utils/schema';
import { getAttachmentPreview } from 'components/Ref';
import { Button } from 'components/Button';
import { DropdownMenu } from 'components/DropdownMenu';
import { CardContainer } from 'Workspace/CardContainer';
import { EraseDocumentConfirmationDialog } from 'Workspace/DocumentEditor/EraseDocumentConfirmationDialog';
import { DocumentViewerHead } from 'Workspace/DocumentEditor/DocumentViewerHead';
import { DocumentEditorForm } from './DocumentEditorForm';

type DocumentEditorProps = {
  document: DocumentDTO;
  onDone: Callback;
  onClone: Callback;
  onErase: Callback;
};

export function DocumentEditor({ document, onDone, onClone, onErase }: DocumentEditorProps) {
  const [showEraseConfirmation, setShowErasetConfirmation] = useState(false);
  const [isDirty, setDirty] = useState(false);

  const formRef = useRef<HTMLFormElement | null>(null);

  return (
    <CardContainer>
      <CardContainer.Topbar
        skipBack={isDirty}
        left={
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
                onClick: () => setShowErasetConfirmation(true),
              },
            ]}
          />
        }
        right={
          isDirty ? (
            <>
              <Button variant="simple" onClick={onDone}>
                Cancel
              </Button>

              <Button
                variant="primary"
                onClick={() => {
                  formRef.current?.requestSubmit();
                }}
              >
                Save
              </Button>
            </>
          ) : (
            <CardContainer.CloseButton />
          )
        }
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
          src="/nothing-to-see-here.jpg"
          alt="funny picture for the erased document"
          className="my-16 mx-auto"
        />
      )}

      <DocumentEditorForm
        formRef={formRef}
        documentId={document.id}
        documentType={document.documentType}
        subtype={document.subtype}
        data={document.data}
        collections={document.collections.map((item) => item.id)}
        onDirty={() => setDirty(true)}
        onSubmit={async (data, subtype, collections) => {
          const submitResult = await RPC.SaveDocument({
            id: document.id,
            subtype,
            data,
            collections,
          });

          if (submitResult.errors) {
            return submitResult.errors;
          }

          onDone();
        }}
      />

      {showEraseConfirmation && (
        <EraseDocumentConfirmationDialog
          documentId={document.id}
          documentType={document.documentType}
          title={document.title}
          onErase={onErase}
          onCancel={() => setShowErasetConfirmation(false)}
        />
      )}
    </CardContainer>
  );
}
