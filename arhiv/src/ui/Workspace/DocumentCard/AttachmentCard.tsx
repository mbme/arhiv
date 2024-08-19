import { useEffect, useState } from 'react';
import { DocumentDTO } from 'dto';
import { useUnsavedChangesWarning } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';
import { DropdownMenu, DropdownOptions } from 'components/DropdownMenu';
import { CardContainer } from 'Workspace/CardContainer';
import { useIsFormDirty } from 'components/Form/Form';
import { AttachmentPreview, canPreview } from 'components/AttachmentPreview';
import { ProgressLocker } from 'components/ProgressLocker';
import { useCardLock } from '../controller';
import { EraseDocumentConfirmationDialog } from '../DocumentEditor/EraseDocumentConfirmationDialog';
import { DocumentViewerHead } from '../DocumentEditor/DocumentViewerHead';
import { DocumentEditor } from '../DocumentEditor/DocumentEditor';
import { useLockDocument } from './useLockDocument';
import { LockError } from './LockError';
import { DocumentTitle } from './DocumentTitle';

type Props = {
  document: DocumentDTO;
  isUpdating: boolean;
  options: DropdownOptions;
};

export function AttachmentCard({ document, isUpdating, options }: Props) {
  const [showEraseConfirmation, setShowErasetConfirmation] = useState(false);

  const [form, setForm] = useState<HTMLFormElement | null>(null);
  const isDirty = useIsFormDirty(form);

  useUnsavedChangesWarning(isDirty);
  useCardLock(isDirty);

  const { lockKey, lockError, resetLockError } = useLockDocument(document.id, isDirty);

  useEffect(() => {
    if (form && lockError) {
      form.reset();
    }
  }, [form, lockError]);

  return (
    <CardContainer
      skipBack={isDirty}
      leftToolbar={
        <DropdownMenu
          icon="dots-horizontal"
          align="bottom-left"
          options={[
            ...options,
            {
              text: `Erase ${document.documentType}`,
              icon: 'erase-document',
              alarming: true,
              onClick: () => {
                setShowErasetConfirmation(true);
              },
            },
          ]}
        />
      }
      title={<DocumentTitle documentType={document.documentType} title={document.title} />}
      showTitleOnScroll
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
        updatedAt={document.updatedAt}
        backrefs={document.backrefs}
      />

      {Boolean(lockError) && (
        <LockError
          error={lockError}
          documentId={document.id}
          documentType={document.documentType}
          onForceUnlock={resetLockError}
        />
      )}

      {canPreview(document.documentType, document.data) && (
        <div className="mb-8 empty:hidden">
          <AttachmentPreview data={document.data} />
        </div>
      )}

      <DocumentEditor
        key={document.updatedAt} // force form fields to use fresh values from the document after save
        formRef={setForm}
        readonlyOverride={lockError ? true : undefined}
        documentType={document.documentType}
        data={document.data}
        collections={document.collections.map((item) => item.id)}
        onSubmit={async (data, collections) => {
          if (!lockKey) {
            throw new Error('lock key is missing');
          }

          const submitResult = await RPC.SaveDocument({
            lockKey,
            id: document.id,
            data,
            collections,
          });

          if (submitResult.errors) {
            return submitResult.errors;
          }
        }}
      />

      {showEraseConfirmation && (
        <EraseDocumentConfirmationDialog
          documentId={document.id}
          documentType={document.documentType}
          title={document.title}
          onCancel={() => {
            setShowErasetConfirmation(false);
          }}
        />
      )}
    </CardContainer>
  );
}
