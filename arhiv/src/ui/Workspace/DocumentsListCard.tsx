import { copyTextToClipbard } from 'utils';
import { useSuspenseQuery } from 'utils/suspense';
import { Ref } from 'components/Ref';
import { showToast } from 'components/Toaster';
import { Button } from 'components/Button';
import { CardContainer } from './CardContainer';
import { Card, useCardContext } from './controller';

type DocumentsListCard = Extract<Card, { variant: 'documents-list' }>;

export function DocumentsListCard() {
  const { card } = useCardContext<DocumentsListCard>();

  const {
    value: { documents },
  } = useSuspenseQuery({
    typeName: 'GetDocuments',
    ids: card.documentIds,
  });

  return (
    <CardContainer title={`Documents list: ${documents.length} documents`}>
      <ol className="pl-4 pt-2 list-decimal">
        {documents.map((document) => (
          <li key={document.id} className="my-4">
            <Button
              className="inline-block mr-6"
              variant="text"
              leadingIcon="clipboard"
              onClick={() => {
                void copyTextToClipbard(document.id).then(() => {
                  showToast({
                    level: 'info',
                    message: 'Copied document id to clipboard!',
                  });
                });
              }}
            >
              {document.id}
            </Button>

            <Ref
              documentId={document.id}
              documentType={document.documentType}
              documentTitle={document.title}
            />
          </li>
        ))}
      </ol>
    </CardContainer>
  );
}
