import { useRef } from 'react';
import { Callback } from '../../../scripts/utils';
import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { Button } from '../Button';
import { CardContainer } from '../CardContainer';
import { Icon } from '../Icon';
import { QueryError } from '../QueryError';
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
    {
      refreshIfChange: [documentId],
    }
  );

  return (
    <>
      <CardContainer.Topbar
        left={
          <span className="section-heading text-lg">{`Edit ${result?.documentType || ''}`}</span>
        }
        right={
          <>
            <Button variant="simple" onClick={onCancel}>
              Cancel
            </Button>

            <Button
              variant="primary"
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

      {inProgress && <Icon variant="spinner" className="mb-8" />}
    </>
  );
}
