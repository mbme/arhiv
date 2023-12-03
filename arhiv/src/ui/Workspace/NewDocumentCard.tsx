import { useRef } from 'react';
import { DEFAULT_SUBTYPE, EMPTY_DATA } from 'dto';
import { RPC } from 'utils/rpc';
import { Button } from 'components/Button';
import { Card, useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';
import { DocumentEditor } from './DocumentEditor/DocumentEditor';

type NewDocumentCard = Extract<Card, { variant: 'new-document' }>;

export function NewDocumentCard() {
  const { card, actions } = useCardContext<NewDocumentCard>();

  const documentType = card.documentType;

  const formRef = useRef<HTMLFormElement | null>(null);

  const onCancel = () => {
    actions.close(card.id);
  };

  return (
    <CardContainer
      leftToolbar={
        <span className="section-heading text-lg">{`New ${documentType || 'document'}`}</span>
      }
      rightToolbar={
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
      skipClose
    >
      <DocumentEditor
        key={documentType}
        autofocus
        formRef={formRef}
        documentType={documentType}
        subtype={card.subtype ?? DEFAULT_SUBTYPE}
        data={card.data ?? EMPTY_DATA}
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

          actions.unlock(card.id);
          actions.replace(card.id, { variant: 'document', documentId: submitResult.id! });
        }}
      />
    </CardContainer>
  );
}
