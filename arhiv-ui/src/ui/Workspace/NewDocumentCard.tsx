import { useRef, useState } from 'preact/hooks';
import { DocumentType } from 'dto';
import { getDocumentTypes, isErasedDocument } from 'utils/schema';
import { RPC } from 'utils/rpc';
import { Button } from 'components/Button';
import { useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';
import { DocumentEditorForm } from './DocumentEditor/DocumentEditorForm';

type NewDocumentCardProps = {
  documentType?: DocumentType;
};
export function NewDocumentCard({ documentType: initialDocumentType }: NewDocumentCardProps) {
  const [documentType, setDocumentType] = useState(initialDocumentType);

  const cardContext = useCardContext();

  const formRef = useRef<HTMLFormElement | null>(null);

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
              variant="primary"
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
          onSubmit={async (data, subtype, collections) => {
            const submitResult = await RPC.CreateDocument({
              documentType,
              subtype,
              data,
              collections,
            });

            if (submitResult.errors) {
              return submitResult.errors;
            }

            cardContext.unlock();
            cardContext.replace({ variant: 'document', documentId: submitResult.id! });
          }}
        />
      ) : (
        <div className="flex justify-around mt-8">
          <section>
            <h1 className="section-heading ml-4">Documents</h1>
            {getDocumentTypes(false).map((documentType) => {
              if (isErasedDocument(documentType)) {
                return null;
              }
              return (
                <Button
                  key={documentType}
                  variant="simple"
                  onClick={() => setDocumentType(documentType)}
                >
                  {documentType}
                </Button>
              );
            })}
          </section>

          <section>
            <h1 className="section-heading ml-4">Collections</h1>
            {getDocumentTypes(true).map((documentType) => (
              <Button
                key={documentType}
                variant="simple"
                onClick={() => setDocumentType(documentType)}
              >
                {documentType}
              </Button>
            ))}
          </section>
        </div>
      )}
    </>
  );
}
