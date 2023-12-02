import { useState } from 'react';
import { DocumentId } from 'dto';
import { isAttachment, isErasedDocument, isProject } from 'utils/schema';
import { useSuspenseQuery } from 'utils/suspense';
import { CardContainer } from 'Workspace/CardContainer';
import { useBazaEvent } from 'Workspace/events';
import { ProgressLocker } from 'components/ProgressLocker';
import { DocumentCard } from './DocumentCard';
import { ErasedDocumentCard } from './ErasedDocumentCard';
import { AttachmentCard } from './AttachmentCard';
import { ProjectCard } from './ProjectCard';

type Props = {
  documentId: DocumentId;
};

export function DocumentCardContainer({ documentId }: Props) {
  const [forceEditor, setForceEditor] = useState(false);

  const {
    value: document,
    isUpdating,
    triggerRefresh,
  } = useSuspenseQuery({ typeName: 'GetDocument', id: documentId });

  useBazaEvent((event) => {
    if (event.typeName === 'Synced') {
      triggerRefresh();
    } else if (event.typeName === 'DocumentUnlocked' && event.id === documentId) {
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

  if (forceEditor) {
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
        onForceEditor={() => setForceEditor(true)}
      />
    );
  }

  return (
    <DocumentCard document={document} isUpdating={isUpdating} triggerRefresh={triggerRefresh} />
  );
}
