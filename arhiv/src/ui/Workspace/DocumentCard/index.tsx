import { isAttachment, isErasedDocument, isProject } from 'utils/schema';
import { useSuspenseQuery } from 'utils/suspense';
import { TASK_DOCUMENT_TYPE } from 'dto';
import { useBazaEvent } from 'baza-events';
import { CardContainer } from 'Workspace/CardContainer';
import { Card, useCardContext } from 'Workspace/workspace-reducer';
import { ProgressLocker } from 'components/ProgressLocker';
import { DocumentCard } from './DocumentCard';
import { ErasedDocumentCard } from './ErasedDocumentCard';
import { AttachmentCard } from './AttachmentCard';
import { ProjectCard } from './ProjectCard';

type DocumentCard = Extract<Card, { variant: 'document' }>;

export function DocumentCardContainer() {
  const { card, actions } = useCardContext<DocumentCard>();

  const {
    value: document,
    isUpdating,
    triggerRefresh,
  } = useSuspenseQuery({ typeName: 'GetDocument', id: card.documentId });

  useBazaEvent((event) => {
    if (event.typeName === 'Synced') {
      triggerRefresh();
    } else if (event.typeName === 'DocumentUnlocked' && event.id === card.documentId) {
      triggerRefresh();
    }
  });

  if (!document) {
    return (
      <CardContainer>
        <ProgressLocker />
      </CardContainer>
    );
  }

  if (card.forceEditor) {
    return (
      <DocumentCard document={document} isUpdating={isUpdating} triggerRefresh={triggerRefresh} />
    );
  }

  if (isErasedDocument(document.documentType)) {
    return <ErasedDocumentCard document={document} isUpdating={isUpdating} />;
  }

  if (isAttachment(document.documentType)) {
    return (
      <AttachmentCard document={document} isUpdating={isUpdating} triggerRefresh={triggerRefresh} />
    );
  }

  if (isProject(document.documentType)) {
    return (
      <ProjectCard
        document={document}
        isUpdating={isUpdating}
        onForceEditor={() =>
          actions.pushStack(card.id, {
            variant: 'document',
            documentId: card.documentId,
            forceEditor: true,
          })
        }
        onAddTask={() => {
          actions.open({
            variant: 'new-document',
            documentType: TASK_DOCUMENT_TYPE,
            collections: [card.documentId],
          });
        }}
      />
    );
  }

  return (
    <DocumentCard document={document} isUpdating={isUpdating} triggerRefresh={triggerRefresh} />
  );
}
