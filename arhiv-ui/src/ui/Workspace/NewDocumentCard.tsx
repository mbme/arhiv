import { useRef } from 'react';
import { DEFAULT_SUBTYPE, DocumentData, DocumentSubtype, DocumentType, EMPTY_DATA } from 'dto';
import { RPC } from 'utils/rpc';
import { Button } from 'components/Button';
import { useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';
import { DocumentEditor } from './DocumentEditor/DocumentEditor';

type NewDocumentCardProps = {
  documentType: DocumentType;
  subtype?: DocumentSubtype;
  data?: DocumentData;
};
export function NewDocumentCard({
  documentType,
  subtype = DEFAULT_SUBTYPE,
  data = EMPTY_DATA,
}: NewDocumentCardProps) {
  const cardContext = useCardContext();

  const formRef = useRef<HTMLFormElement | null>(null);

  const onCancel = () => {
    cardContext.close();
  };

  return (
    <CardContainer>
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

      <DocumentEditor
        key={documentType}
        autofocus
        formRef={formRef}
        documentType={documentType}
        subtype={subtype}
        data={data}
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
    </CardContainer>
  );
}
