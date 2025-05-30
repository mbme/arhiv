import { useEffect, useState } from 'react';
import { DocumentDTO } from 'dto';
import { useUnsavedChangesWarning } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';
import { DropdownMenu, DropdownOptions } from 'components/DropdownMenu';
import { CardContainer } from 'Workspace/CardContainer';
import { useIsFormDirty } from 'components/Form/Form';
import { ProgressLocker } from 'components/ProgressLocker';
import { dispatchDocumentChangeEvent } from 'Workspace/documentChangeUtils';
import { useCardContext, useCardLock } from '../controller';
import { EraseDocumentConfirmationDialog } from '../DocumentEditor/EraseDocumentConfirmationDialog';
import { DocumentViewerHead } from './DocumentViewerHead';
import { DocumentEditor } from '../DocumentEditor/DocumentEditor';
import { useLockDocument } from './useLockDocument';
import { LockError } from './LockError';
import { DocumentTitle } from './DocumentTitle';
import { CONFLICT_INDICATOR, STAGED_INDICATOR } from './Indicators';

type Props = {
  document: DocumentDTO;
  isUpdating: boolean;
  options: DropdownOptions;
};

export function DocumentCard({ document, isUpdating, options }: Props) {
  const { controller } = useCardContext();
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
        <>
          <DropdownMenu
            icon="dots-horizontal"
            align="bottom-left"
            options={[
              ...options,
              {
                text: `Clone ${document.documentType}`,
                icon: 'duplicate-document',
                onClick: () => {
                  controller.newDocument(document.documentType, document.data);
                },
              },
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

          {document.isStaged && STAGED_INDICATOR}
          {document.hasConflict && CONFLICT_INDICATOR}
        </>
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
        snapshotsCount={document.snapshotsCount}
      />

      {Boolean(lockError) && (
        <LockError
          error={lockError}
          documentId={document.id}
          documentType={document.documentType}
          onForceUnlock={resetLockError}
        />
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

          dispatchDocumentChangeEvent([document.id]);
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
