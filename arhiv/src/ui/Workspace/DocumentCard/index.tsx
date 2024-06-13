import { isAttachment, isErasedDocument, isProject } from 'utils/schema';
import { useSuspenseQuery } from 'utils/suspense';
import { copyTextToClipbard, getDocumentUrl } from 'utils';
import { TASK_DOCUMENT_TYPE } from 'dto';
import { useBazaEvent } from 'baza-events';
import { CardContainer } from 'Workspace/CardContainer';
import { Card, useCardContext } from 'Workspace/workspace-reducer';
import { ProgressLocker } from 'components/ProgressLocker';
import { DropdownOptions } from 'components/DropdownMenu';
import { showToast } from 'components/Toaster';
import { DocumentCard } from './DocumentCard';
import { ErasedDocumentCard } from './ErasedDocumentCard';
import { AttachmentCard } from './AttachmentCard';
import { ProjectCard } from './ProjectCard';

type DocumentCard = Extract<Card, { variant: 'document' }>;

export function DocumentCardContainer() {
  const { card, controller } = useCardContext<DocumentCard>();

  const {
    value: document,
    isUpdating,
    triggerRefresh,
  } = useSuspenseQuery({ typeName: 'GetDocument', id: card.documentId });

  useBazaEvent((event) => {
    if (event.typeName === 'Synced') {
      triggerRefresh();
    } else if (event.typeName === 'DocumentStaged' && event.id === card.documentId) {
      triggerRefresh();
    } else if (event.typeName === 'DocumentStaged') {
      if (
        document.refs.includes(event.id) ||
        document.backrefs.some((backref) => backref.id === event.id) ||
        document.collections.some((backref) => backref.id === event.id)
      ) {
        triggerRefresh(true);
      }
    }
  });

  if (!document) {
    return (
      <CardContainer>
        <ProgressLocker />
      </CardContainer>
    );
  }

  const documentActions: DropdownOptions = [
    {
      text: `ID ${document.id}`,
      icon: 'clipboard',
      onClick: () => {
        void copyTextToClipbard(document.id).then(() => {
          showToast({
            level: 'info',
            message: 'Copied document id to clipboard!',
          });
        });
      },
    },
    {
      text: 'Copy link',
      icon: 'clipboard',
      onClick: () => {
        void copyTextToClipbard(getDocumentUrl(document.id)).then(() => {
          showToast({
            level: 'info',
            message: 'Copied document url to clipboard!',
          });
        });
      },
    },
  ];

  if (card.forceEditor) {
    return <DocumentCard document={document} isUpdating={isUpdating} options={documentActions} />;
  }

  if (isErasedDocument(document.documentType)) {
    return <ErasedDocumentCard document={document} isUpdating={isUpdating} />;
  }

  if (isAttachment(document.documentType)) {
    return <AttachmentCard document={document} isUpdating={isUpdating} options={documentActions} />;
  }

  if (isProject(document.documentType)) {
    return (
      <ProjectCard
        document={document}
        isUpdating={isUpdating}
        onForceEditor={() =>
          controller.pushStack(card.id, {
            variant: 'document',
            documentId: card.documentId,
            forceEditor: true,
          })
        }
        onAddTask={() => {
          controller.open({
            variant: 'new-document',
            documentType: TASK_DOCUMENT_TYPE,
            collections: [card.documentId],
          });
        }}
        options={documentActions}
      />
    );
  }

  return <DocumentCard document={document} isUpdating={isUpdating} options={documentActions} />;
}
