import { useState } from 'react';
import { DocumentId, DocumentType } from 'dto';
import { Callback } from 'utils';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/network';
import { Button } from 'components/Button';
import { QueryError } from 'components/QueryError';
import { Dialog } from 'components/Dialog';
import { showToast } from 'components/Toaster';

type Props = {
  documentId: DocumentId;
  documentType: DocumentType;
  error: unknown;
  onForceUnlock: Callback;
};
export function LockError({ documentId, documentType, error: lockError, onForceUnlock }: Props) {
  const [showConfirmation, setShowConfirmation] = useState(false);

  const {
    error: unlockError,
    inProgress,
    triggerRefresh,
  } = useQuery(
    async (abortSignal) => {
      await RPC.UnlockDocument({ id: documentId, forceUnlock: true }, abortSignal);
      setShowConfirmation(false);
      showToast({ level: 'info', message: `Unlocked document ${documentId}` });
      onForceUnlock();
    },
    {
      refreshOnMount: false,
    },
  );

  const hideModal = () => {
    if (inProgress) {
      return;
    }

    setShowConfirmation(false);
  };

  const buttons = (
    <>
      <Button variant="simple" onClick={hideModal} disabled={inProgress}>
        Cancel
      </Button>

      <Button variant="primary" alarming onClick={triggerRefresh} busy={inProgress}>
        UNLOCK
      </Button>
    </>
  );

  return (
    <>
      <QueryError error={`Failed to lock ${documentType || 'document'}: ${String(lockError)}`}>
        <Button
          variant="text"
          alarming
          onClick={() => {
            setShowConfirmation(true);
          }}
        >
          Unlock
        </Button>

        {showConfirmation && (
          <Dialog
            onHide={hideModal}
            alarming
            title={`Unlock ${documentType || 'document'}`}
            buttons={buttons}
          >
            Do you really want to force unlock the {documentType || 'document'} <b>{documentId}</b>?
          </Dialog>
        )}
      </QueryError>

      {unlockError && (
        <QueryError error={`Failed to force unlock document: ${String(unlockError)}`} />
      )}
    </>
  );
}
