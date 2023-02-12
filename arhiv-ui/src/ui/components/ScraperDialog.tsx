import { useState } from 'preact/hooks';
import { DocumentId } from 'dto';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';
import { QueryError } from 'components/QueryError';

type Props = {
  onSuccess: (url: string, ids: DocumentId[]) => void;
  onCancel: () => void;
};
export function ScraperDialog({ onSuccess, onCancel }: Props) {
  const [url, setUrl] = useState('');

  const { error, inProgress, triggerRefresh } = useQuery(
    async (abortSignal) => RPC.Scrape({ url }, abortSignal),
    {
      refreshOnMount: false,
      onSuccess(result) {
        onSuccess(
          url,
          result.documents.map((document) => document.id)
        );
      },
    }
  );

  return (
    <Dialog onHide={onCancel} title="Scrape URL">
      <form
        className="modal-content form"
        onSubmit={(e) => {
          e.preventDefault();
          triggerRefresh();
        }}
      >
        <div className="flex gap-2 mb-8">
          <input
            type="url"
            name="url"
            placeholder="Enter URL"
            className="field grow"
            value={url}
            onChange={(e) => setUrl(e.currentTarget.value)}
            disabled={inProgress}
            autoComplete="off"
            autofocus
          />

          <Button type="submit" variant="primary" busy={inProgress}>
            Scrape!
          </Button>
        </div>

        {error && <QueryError error={error} />}
      </form>
    </Dialog>
  );
}
