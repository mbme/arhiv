import { useRef } from 'react';
import { DocumentId, DocumentType } from 'dto';
import { Callback } from 'utils';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { Button } from 'components/Button';
import { Dialog } from 'components/Dialog';
import { QueryError } from 'components/QueryError';

type EraseDocumentButtonProps = {
  documentId: DocumentId;
  documentType: DocumentType;
  title: string;
  onErase: Callback;
  onCancel: Callback;
};
export function EraseDocumentConfirmationDialog({
  documentId,
  documentType,
  title,
  onErase,
  onCancel,
}: EraseDocumentButtonProps) {
  const formRef = useRef<HTMLFormElement | null>(null);

  const { error, inProgress, triggerRefresh } = useQuery(
    async (abortSignal) => {
      await RPC.EraseDocument({ id: documentId }, abortSignal);
      hideModal();
      onErase();
    },
    {
      refreshOnMount: false,
    }
  );

  const hideModal = () => {
    if (inProgress) {
      return;
    }

    onCancel();
  };

  const confirmationText = `erase ${documentType}`;

  const buttons = (
    <>
      <Button variant="simple" onClick={hideModal} disabled={inProgress}>
        Cancel
      </Button>

      <Button
        variant="primary"
        alarming
        onClick={() => formRef.current?.requestSubmit()}
        busy={inProgress}
      >
        ERASE
      </Button>
    </>
  );

  return (
    <Dialog onHide={hideModal} alarming title={`Erase ${documentType}`} buttons={buttons}>
      <form
        className="form"
        ref={formRef}
        onSubmit={(e) => {
          e.preventDefault();
          triggerRefresh();
        }}
      >
        Do you really want to erase the {documentType} <b>{title}</b> and its history?
        <label className="block mt-8">
          Type <b>{confirmationText}</b> to confirm:
          <input
            type="text"
            autoComplete="off"
            name="confirmation_text"
            className="field mt-2"
            required
            pattern={confirmationText}
            defaultValue=""
            disabled={inProgress}
          />
        </label>
        {error && <QueryError error={error} />}
      </form>
    </Dialog>
  );
}
