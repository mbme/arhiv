import { useState } from 'preact/hooks';
import { useQuery } from '../utils/hooks';
import { RPC } from '../utils/rpc';
import { Button } from '../components/Button';
import { QueryError } from '../components/QueryError';
import { Ref } from '../components/Ref';
import { CardContainer } from './CardContainer';
import { useCardContext } from './workspace-reducer';

export function ScraperCard() {
  const { open } = useCardContext();

  const [url, setUrl] = useState('');

  const { result, error, inProgress, triggerRefresh } = useQuery(
    async (abortSignal) => RPC.Scrape({ url }, abortSignal),
    {
      refreshOnMount: false,
    }
  );

  return (
    <>
      <CardContainer.Topbar
        left={<span className="section-heading text-lg">Scrape URL</span>}
        right={<CardContainer.CloseButton />}
      />

      {!result && (
        <form
          className="form"
          onSubmit={(e) => {
            e.preventDefault();
            triggerRefresh();
          }}
        >
          <div className="flex gap-2 mb-4 p-1">
            <input
              type="url"
              name="url"
              placeholder="Enter URL"
              className="field grow"
              value={url}
              onChange={(e) => setUrl(e.currentTarget.value)}
              disabled={inProgress}
              autoComplete="off"
            />

            <Button type="submit" variant="primary" busy={inProgress}>
              Scrape!
            </Button>
          </div>

          {error && <QueryError error={error} />}
        </form>
      )}

      {result && (
        <>
          <h1>{url}</h1>

          {result.documents.map((document) => (
            <div key={document.id} className="mb-4">
              <Ref
                documentType={document.documentType}
                subtype={document.subtype}
                documentTitle={document.title}
                onClick={() => open({ variant: 'document', documentId: document.id })}
              />
            </div>
          ))}
        </>
      )}
    </>
  );
}
