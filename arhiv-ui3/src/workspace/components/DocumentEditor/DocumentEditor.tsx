import { Callback } from '../../../scripts/utils';
import { useQuery } from '../../hooks';
import { RPC } from '../../rpc';
import { Form } from '../Form/Form';
import { QueryError } from '../QueryError';
import { DocumentEditorFields } from './DocumentEditorFields';

type DocumentEditorProps = {
  documentId: string;
  onSave: Callback;
  onCancel: Callback;
};

export function DocumentEditor({ documentId, onSave, onCancel }: DocumentEditorProps) {
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    []
  );

  return (
    <div className="p-8">
      {result && (
        <div className="flex gap-2 justify-between bg-neutral-200 py-2 mb-12 sticky top-0 z-10">
          <button className="font-mono" onClick={onCancel}>
            CANCEL
          </button>

          <button className="font-mono" onClick={onSave}>
            SAVE
          </button>
        </div>
      )}

      <QueryError error={error} />

      {inProgress && <div className="mb-8">Loading...</div>}

      {result && (
        <Form>
          <DocumentEditorFields
            documentType={result.documentType}
            subtype={result.subtype}
            data={result.data}
          />
        </Form>
      )}
    </div>
  );
}
