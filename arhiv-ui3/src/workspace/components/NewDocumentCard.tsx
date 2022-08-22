import { useRef } from 'preact/hooks';
import { useCardContext } from '../workspace-reducer';
import { Button } from './Button';
import { CardTopbar } from './CardTopbar';
import { DocumentEditorForm } from './DocumentEditor/DocumentEditorForm';

type NewDocumentCardProps = {
  documentType: string;
};
export function NewDocumentCard({ documentType }: NewDocumentCardProps) {
  const cardContext = useCardContext();

  const formRef = useRef<HTMLFormElement | null>(null);

  const onSave = (documentId: string) => {
    cardContext.replace({ variant: 'document', documentId });
  };
  const onCancel = () => {
    cardContext.close();
  };

  return (
    <>
      <CardTopbar>
        <Button variant="simple" onClick={onCancel}>
          Cancel
        </Button>

        <Button
          variant="prime"
          onClick={() => {
            formRef.current?.requestSubmit();
          }}
        >
          Create
        </Button>
      </CardTopbar>

      <DocumentEditorForm formRef={formRef} documentType={documentType} onSave={onSave} />
    </>
  );
}
