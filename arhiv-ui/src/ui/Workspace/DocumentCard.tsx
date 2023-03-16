import { useState } from 'react';
import { DocumentId } from 'dto';
import { copyTextToClipbard, getDocumentUrl } from 'utils';
import { useQuery, useUnsavedChangesWarning } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { isAttachment, isErasedDocument } from 'utils/schema';
import { QueryError } from 'components/QueryError';
import { Icon } from 'components/Icon';
import { Button } from 'components/Button';
import { DropdownMenu } from 'components/DropdownMenu';
import { CardContainer } from 'Workspace/CardContainer';
import { useIsFormDirty } from 'components/Form/Form';
import { getAttachmentPreview } from 'components/Ref';
import { ProgressLocker } from 'components/ProgressLocker';
import { useCardContext, useCardLock } from './workspace-reducer';
import { EraseDocumentConfirmationDialog } from './DocumentEditor/EraseDocumentConfirmationDialog';
import { DocumentViewerHead } from './DocumentEditor/DocumentViewerHead';
import { DocumentEditor } from './DocumentEditor/DocumentEditor';

type Props = {
  documentId: DocumentId;
};

export function DocumentCard({ documentId }: Props) {
  const context = useCardContext();

  const [showEraseConfirmation, setShowErasetConfirmation] = useState(false);

  const [form, setForm] = useState<HTMLFormElement | null>(null);
  const isDirty = useIsFormDirty(form);

  useUnsavedChangesWarning(isDirty);
  useCardLock(isDirty);

  const {
    result: document,
    error,
    inProgress,
    triggerRefresh,
  } = useQuery((abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal), {
    refreshIfChange: [documentId],
  });

  if (error) {
    return (
      <CardContainer>
        <QueryError error={error} />
      </CardContainer>
    );
  }

  if (!document) {
    return (
      <CardContainer>
        <Icon variant="spinner" className="mb-8" />
      </CardContainer>
    );
  }

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
                onClick: () => {
                  context.open({
                    variant: 'new-document',
                    documentType: document.documentType,
                    subtype: document.subtype,
                    data: document.data,
                  });
                },
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
              <Button
                variant="simple"
                onClick={() => {
                  form?.reset();
                }}
              >
                Cancel
              </Button>

              <Button
                variant="primary"
                onClick={() => {
                  form?.requestSubmit();
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

      {inProgress && <ProgressLocker />}

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

      <DocumentEditor
        formRef={setForm}
        documentId={document.id}
        documentType={document.documentType}
        subtype={document.subtype}
        data={document.data}
        collections={document.collections.map((item) => item.id)}
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

          triggerRefresh();
        }}
      />

      {showEraseConfirmation && (
        <EraseDocumentConfirmationDialog
          documentId={document.id}
          documentType={document.documentType}
          title={document.title}
          onErase={triggerRefresh}
          onCancel={() => setShowErasetConfirmation(false)}
        />
      )}
    </CardContainer>
  );
}
