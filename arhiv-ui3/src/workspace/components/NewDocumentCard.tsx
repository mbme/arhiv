import { useRef, useState } from 'preact/hooks';
import { getDocumentTypes } from '../schema';
import { useCardContext } from '../workspace-reducer';
import { Button } from './Button';
import { CardContainer } from './CardContainer';
import { DocumentEditorForm } from './DocumentEditor/DocumentEditorForm';

type NewDocumentCardProps = {
  documentType?: string;
};
export function NewDocumentCard({ documentType: initialDocumentType }: NewDocumentCardProps) {
  const [documentType, setDocumentType] = useState(initialDocumentType);

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
      <CardContainer.Topbar
        left={
          <span className="section-heading text-lg">{`New ${documentType || 'document'}`}</span>
        }
        right={
          <>
            <Button variant="simple" onClick={onCancel}>
              Cancel
            </Button>

            <Button
              variant="prime"
              disabled={!documentType}
              onClick={() => {
                formRef.current?.requestSubmit();
              }}
            >
              Create
            </Button>
          </>
        }
      />

      {documentType ? (
        <DocumentEditorForm
          key={documentType}
          formRef={formRef}
          documentType={documentType}
          onSave={onSave}
        />
      ) : (
        <div className="flex flex-wrap gap-4 px-8 py-16">
          {getDocumentTypes().map((documentType) => (
            <Button
              key={documentType}
              variant="simple"
              onClick={() => setDocumentType(documentType)}
            >
              {documentType}
            </Button>
          ))}
        </div>
      )}
    </>
  );
}
