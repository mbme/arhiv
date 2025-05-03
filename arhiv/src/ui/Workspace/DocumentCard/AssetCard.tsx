import { useEffect, useState } from 'react';
import { DocumentDTO } from 'dto';
import { useUnsavedChangesWarning } from 'utils/hooks';
import { getAssetUrl, RPC } from 'utils/network';
import { Button } from 'components/Button';
import { DropdownMenu, DropdownOptions } from 'components/DropdownMenu';
import { CardContainer } from 'Workspace/CardContainer';
import { useIsFormDirty } from 'components/Form/Form';
import { AssetPreview, canPreview } from 'components/AssetPreview';
import { ProgressLocker } from 'components/ProgressLocker';
import { DownloadLink } from 'components/Link';
import { dispatchDocumentChangeEvent } from 'Workspace/documentChangeUtils';
import { useCardLock } from '../controller';
import { EraseDocumentConfirmationDialog } from '../DocumentEditor/EraseDocumentConfirmationDialog';
import { DocumentViewerHead } from './DocumentViewerHead';
import { DocumentEditor } from '../DocumentEditor/DocumentEditor';
import { useLockDocument } from './useLockDocument';
import { LockError } from './LockError';
import { DocumentTitle } from './DocumentTitle';
import { CONFLICT_INDICATOR } from './ConflictIndicator';

type Props = {
  document: DocumentDTO;
  isUpdating: boolean;
  options: DropdownOptions;
};

export function AssetCard({ document, isUpdating, options }: Props) {
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

  const filename = document.data['filename'] as string;

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
                text: `Erase ${document.documentType}`,
                icon: 'erase-document',
                alarming: true,
                onClick: () => {
                  setShowErasetConfirmation(true);
                },
              },
            ]}
          />

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

      {canPreview(document.documentType, document.data) && (
        <div className="mb-8 empty:hidden">
          <AssetPreview assetId={document.id} data={document.data} />
        </div>
      )}

      <DownloadLink
        url={getAssetUrl(document.id)}
        fileName={filename}
        title="Download file"
        className="justify-self-end"
      />

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
