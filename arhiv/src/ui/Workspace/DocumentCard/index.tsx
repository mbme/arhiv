import { useState } from 'react';
import { DocumentId } from 'dto';
import { copyTextToClipbard, getDocumentUrl } from 'utils';
import { useUnsavedChangesWarning } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { isAttachment, isErasedDocument } from 'utils/schema';
import { useSuspenseQuery } from 'utils/suspense';
import { Button } from 'components/Button';
import { DropdownMenu } from 'components/DropdownMenu';
import { CardContainer } from 'Workspace/CardContainer';
import { useIsFormDirty } from 'components/Form/Form';
import { getAttachmentPreview } from 'components/Ref';
import { ProgressLocker } from 'components/ProgressLocker';
import { useCardContext, useCardLock } from '../workspace-reducer';
import { EraseDocumentConfirmationDialog } from '../DocumentEditor/EraseDocumentConfirmationDialog';
import { DocumentViewerHead } from '../DocumentEditor/DocumentViewerHead';
import { DocumentEditor } from '../DocumentEditor/DocumentEditor';
import { useLockDocument } from './useLockDocument';

type Props = {
  documentId: DocumentId;
};

export function DocumentCard({ documentId }: Props) {
  const { actions } = useCardContext();

  const [showEraseConfirmation, setShowErasetConfirmation] = useState(false);

  const [form, setForm] = useState<HTMLFormElement | null>(null);
  const isDirty = useIsFormDirty(form);

  useUnsavedChangesWarning(isDirty);
  useCardLock(isDirty);

  const lockKey = useLockDocument(documentId, isDirty);

  const {
    value: document,
    isUpdating,
    triggerRefresh,
  } = useSuspenseQuery({ typeName: 'GetDocument', id: documentId });

  return (
    <CardContainer
      skipBack={isDirty}
      leftToolbar={
        <DropdownMenu
          icon="dots-horizontal"
          align="bottom-left"
          options={[
            {
              text: `ID ${document.id}`,
              icon: 'clipboard',
              onClick: () => {
                void copyTextToClipbard(document.id);
              },
            },
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
                actions.open({
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
      rightToolbar={
        isDirty && (
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
        )
      }
      skipClose={isDirty}
    >
      {isUpdating && <ProgressLocker />}

      <DocumentViewerHead
        documentType={document.documentType}
        subtype={document.subtype}
        updatedAt={document.updatedAt}
        backrefs={document.backrefs}
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
          if (!lockKey) {
            throw new Error('lock key is missing');
          }

          const submitResult = await RPC.SaveDocument({
            lockKey,
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
