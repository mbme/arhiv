import { useRef, useState } from 'preact/hooks';
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
  const formRef = useRef<HTMLFormElement>(null);
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
        <Dialog
          onHide={hideModal}
          variant="warn"
          title={`Erase ${documentType}`}
          buttons={
            <>
              <Button variant="simple" className="mr-8" onClick={hideModal} disabled={inProgress}>
                Cancel
              </Button>

              <Button
                variant="prime"
                color="danger"
                loading={inProgress}
                onClick={() => formRef.current?.requestSubmit()}
              >
                ERASE
              </Button>
            </>
          }
        >
          <form
            ref={formRef}
            className="form"
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
          </form>
        </Dialog>
      )}
    </Button>
  );
}
