import { useRef } from 'preact/hooks';
import { CardContext, useCardContext } from '../workspace-reducer';
import { Button } from './Button';
import { CardContainer } from './CardContainer';
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
      <CardContainer.Topbar>
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
      </CardContainer.Topbar>

      <DocumentEditorForm formRef={formRef} documentType={documentType} onSave={onSave} />
    </>
  );
}
