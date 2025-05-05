import { copyTextToClipbard } from 'utils';
import { useSuspenseQuery } from 'utils/suspense';
import { createLink, createRefUrl } from 'utils/markup';
import { Ref } from 'components/Ref';
import { showToast } from 'components/Toaster';
import { Button } from 'components/Button';
import { CardContainer } from './CardContainer';
import { Card, useCardContext } from './controller';
import { useDocumentChange } from './documentChangeUtils';

type DocumentsListCard = Extract<Card, { variant: 'documents-list' }>;

export function DocumentsListCard() {
  const { card } = useCardContext<DocumentsListCard>();

  const {
    value: { documents },
    triggerRefresh,
  } = useSuspenseQuery({
    typeName: 'GetDocuments',
    ids: card.documentIds,
  });

  useDocumentChange(
    documents.map((document) => document.id),
    () => {
      triggerRefresh(true);
    },
  );

  return (
    <CardContainer title={`${card.title}: ${documents.length} documents`}>
      <ol className="pl-4 pt-2 list-decimal">
        {documents.map((document) => (
          <li key={document.id} className="my-4">
            <Button
              className="inline-block mr-6"
              variant="text"
              leadingIcon="clipboard"
              onClick={() => {
                void copyTextToClipbard(createLink(createRefUrl(document.id), '')).then(() => {
                  showToast({
                    level: 'info',
                    message: 'Copied document ref to clipboard!',
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

      <Button
        variant="primary"
        onClick={() => {
          const refList = documents
            .map((document) => '* ' + createLink(createRefUrl(document.id), ''))
            .join('\n');

          void copyTextToClipbard(refList).then(() => {
            showToast({
              level: 'info',
              message: 'Copied list of refs to clipboard!',
            });
          });
        }}
      >
        Copy ref list
      </Button>
    </CardContainer>
  );
}
