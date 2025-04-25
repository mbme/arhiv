import { isAsset, isErasedDocument, isProject } from 'utils/schema';
import { useSuspenseQuery } from 'utils/suspense';
import { copyTextToClipbard } from 'utils';
import { getDocumentUrl } from 'utils/network';
import { TASK_DOCUMENT_TYPE } from 'dto';
import { Card, useCardContext } from 'Workspace/controller';
import { DropdownOptions } from 'components/DropdownMenu';
import { showToast } from 'components/Toaster';
import { useDocumentChangeHandler } from 'Workspace/documentChangeUtils';
import { DocumentCard } from './DocumentCard';
import { ErasedDocumentCard } from './ErasedDocumentCard';
import { AssetCard } from './AssetCard';
import { ProjectCard } from './ProjectCard';

type DocumentCard = Extract<Card, { variant: 'document' }>;

export function DocumentCardContainer() {
  const { card, controller } = useCardContext<DocumentCard>();

  const {
    value: document,
    isUpdating,
    triggerRefresh,
  } = useSuspenseQuery({
    typeName: 'GetDocument',
    id: card.documentId,
  });

  // refresh document if referenced document changes
  useDocumentChangeHandler((updatedDocumentsIds) => {
    const referencedDocumentIds = new Set([
      document.id,
      ...document.refs,
      ...document.backrefs.map((item) => item.id),
      ...document.collections.map((item) => item.id),
    ]);

    const someReferencedDocumentsUpdated = [...referencedDocumentIds].some((id) =>
      updatedDocumentsIds.has(id),
    );

    if (someReferencedDocumentsUpdated) {
      triggerRefresh(true);
    }
  });

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
  ];

  if (card.forceEditor) {
    return <DocumentCard document={document} isUpdating={isUpdating} options={documentActions} />;
  }

  if (isErasedDocument(document.documentType)) {
    return <ErasedDocumentCard document={document} isUpdating={isUpdating} />;
  }

  if (isAsset(document.documentType)) {
    return <AssetCard document={document} isUpdating={isUpdating} options={documentActions} />;
  }

  if (isProject(document.documentType)) {
    return (
      <ProjectCard
        document={document}
        isUpdating={isUpdating}
        onForceEditor={() => {
          controller.pushStack(card.id, {
            variant: 'document',
            documentId: card.documentId,
            forceEditor: true,
          });
        }}
        onAddTask={() => {
          controller.newDocument(TASK_DOCUMENT_TYPE, undefined, [card.documentId]);
        }}
        options={documentActions}
      />
    );
  }

  return <DocumentCard document={document} isUpdating={isUpdating} options={documentActions} />;
}
