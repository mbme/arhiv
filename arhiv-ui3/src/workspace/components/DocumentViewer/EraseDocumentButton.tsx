import { useState } from 'preact/hooks';
import { Callback } from '../../../scripts/utils';
import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { Button } from '../Button';
import { Dialog } from '../Dialog';
import { QueryError } from '../QueryError';

type EraseDocumentButtonProps = {
  documentId: string;
  documentType: string;
  title: string;
  onErase: Callback;
};
export function EraseDocumentButton({
  documentId,
  documentType,
  title,
  onErase,
}: EraseDocumentButtonProps) {
  const [showModal, setShowModal] = useState(false);

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

    setShowModal(false);
  };

  const confirmationText = `erase ${documentType}`;

  return (
    <Button variant="text" color="warn" icon="erase-document" onClick={() => setShowModal(true)}>
      Erase
      {showModal && (
        <Dialog onHide={hideModal}>
          <form
            className="form"
            onSubmit={(e) => {
              e.preventDefault();
              triggerRefresh();
            }}
          >
            <div className="modal-title bg-red-500 bg-opacity-75">Erase {documentType}</div>

            <div className="modal-content">
              Do you really want to erase the {documentType} <b>{title}</b> and its history?
              <label className="block mt-8">
                Type <b>{confirmationText}</b> to confirm:
                <input
                  type="text"
                  autocomplete="off"
                  name="confirmation_text"
                  className="field mt-2"
                  required
                  pattern={confirmationText}
                  value=""
                  disabled={inProgress}
                />
              </label>
              {error && <QueryError error={error} />}
            </div>

            <div className="modal-buttons">
              <Button variant="simple" className="mr-8" onClick={hideModal} disabled={inProgress}>
                Cancel
              </Button>

              <Button type="submit" variant="prime" color="danger" loading={inProgress}>
                ERASE
              </Button>
            </div>
          </form>
        </Dialog>
      )}
    </Button>
  );
}
