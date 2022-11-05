import { Callback } from 'utils';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { Button } from 'components/Button';
import { Dialog } from 'components/Dialog';
import { QueryError } from 'components/QueryError';

type EraseDocumentButtonProps = {
  documentId: string;
  documentType: string;
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

  return (
    <Dialog onHide={hideModal} alarming title={`Erase ${documentType}`}>
      <form
        className="form"
        onSubmit={(e) => {
          e.preventDefault();
          triggerRefresh();
        }}
      >
        <div className="modal-content">
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
        </div>

        <div className="modal-buttons">
          <Button variant="simple" onClick={hideModal} disabled={inProgress}>
            Cancel
          </Button>

          <Button variant="primary" alarming type="submit" busy={inProgress}>
            ERASE
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
