import { useRef } from 'preact/hooks';
import { Callback } from '../../../scripts/utils';
import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { Button } from '../Button';
import { CardContainer } from '../CardContainer';
import { QueryError } from '../QueryError';
import { Spinner } from '../Spinner';
import { DocumentEditorForm } from './DocumentEditorForm';

type DocumentEditorProps = {
  documentId: string;
  onSave: Callback;
  onCancel: Callback;
};

export function DocumentEditor({ documentId, onSave, onCancel }: DocumentEditorProps) {
  const formRef = useRef<HTMLFormElement | null>(null);

  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    [documentId]
  );

  return (
    <>
      <CardContainer.Topbar
        title={`Edit ${result?.documentType || ''}`}
        right={
          <>
            <Button variant="simple" onClick={onCancel}>
              Cancel
            </Button>

            <Button
              variant="prime"
              onClick={() => {
                formRef.current?.requestSubmit();
              }}
              disabled={!result}
            >
              Save
            </Button>
          </>
        }
      />

      {result && (
        <DocumentEditorForm
          formRef={formRef}
          documentId={documentId}
          documentType={result.documentType}
          subtype={result.subtype}
          data={result.data}
          onSave={onSave}
        />
      )}

      {error && <QueryError error={error} />}

      {inProgress && (
        <div className="mb-8">
          <Spinner /> Loading...
        </div>
      )}
    </>
  );
}
