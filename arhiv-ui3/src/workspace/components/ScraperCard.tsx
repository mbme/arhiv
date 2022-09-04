import { useState } from 'react';
import { useQuery } from '../hooks';
import { RPC } from '../rpc';
import { Button } from './Button';
import { CardContainer } from './CardContainer';
import { QueryError } from './QueryError';
import { Ref } from './Ref';

export function ScraperCard() {
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
          <div className="flex mb-4">
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
                id={document.id}
                documentType={document.documentType}
                subtype={document.subtype}
                title={document.title}
              />
            </div>
          ))}
        </>
      )}
    </>
  );
}
