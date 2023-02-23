import { useRef } from 'preact/hooks';
import { DocumentDTO } from 'dto';
import { Callback } from 'utils';
import { RPC } from 'utils/rpc';
import { Button } from 'components/Button';
import { CardContainer } from '../CardContainer';
import { DocumentEditorForm } from './DocumentEditorForm';

type DocumentEditorProps = {
  document: DocumentDTO;
  onSave: Callback;
  onCancel: Callback;
};

export function DocumentEditor({ document, onSave, onCancel }: DocumentEditorProps) {
  const formRef = useRef<HTMLFormElement | null>(null);

  return (
    <CardContainer>
      <CardContainer.Topbar
        skipBack
        left={
          <span className="section-heading text-lg">{`Edit ${document.documentType || ''}`}</span>
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
            >
              Save
            </Button>
          </>
        }
      />

      <DocumentEditorForm
        formRef={formRef}
        documentId={document.id}
        documentType={document.documentType}
        subtype={document.subtype}
        data={document.data}
        collections={document.collections.map((item) => item.id)}
        onSubmit={async (data, subtype, collections) => {
          const submitResult = await RPC.SaveDocument({
            id: document.id,
            subtype,
            data,
            collections,
          });

          if (submitResult.errors) {
            return submitResult.errors;
          }

          onSave();
        }}
      />
    </CardContainer>
  );
}
