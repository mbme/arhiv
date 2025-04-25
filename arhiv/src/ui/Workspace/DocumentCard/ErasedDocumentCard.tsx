import { DocumentDTO } from 'dto';
import { CardContainer } from 'Workspace/CardContainer';
import { ProgressLocker } from 'components/ProgressLocker';
import { DocumentViewerHead } from './DocumentViewerHead';
import { CONFLICT_INDICATOR } from './ConflictIndicator';

type Props = {
  document: DocumentDTO;
  isUpdating: boolean;
};

export function ErasedDocumentCard({ document, isUpdating }: Props) {
  return (
    <CardContainer leftToolbar={document.hasConflict && CONFLICT_INDICATOR} title="ERASED DOCUMENT">
      {isUpdating && <ProgressLocker />}

      <DocumentViewerHead
        id={document.id}
        documentType={document.documentType}
        updatedAt={document.updatedAt}
        backrefs={document.backrefs}
        snapshotsCount={document.snapshotsCount}
      />

      <img
        src={`${window.CONFIG.basePath}/nothing-to-see-here.jpg`}
        alt="funny picture for the erased document"
        className="my-16 mx-auto"
      />
    </CardContainer>
  );
}
