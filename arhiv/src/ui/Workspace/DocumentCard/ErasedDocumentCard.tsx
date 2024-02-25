import { DocumentDTO } from 'dto';
import { CardContainer } from 'Workspace/CardContainer';
import { ProgressLocker } from 'components/ProgressLocker';
import { DocumentViewerHead } from '../DocumentEditor/DocumentViewerHead';

type Props = {
  document: DocumentDTO;
  isUpdating: boolean;
};

export function ErasedDocumentCard({ document, isUpdating }: Props) {
  return (
    <CardContainer>
      {isUpdating && <ProgressLocker />}

      <DocumentViewerHead
        id={document.id}
        documentType={document.documentType}
        updatedAt={document.updatedAt}
        backrefs={document.backrefs}
      />

      <img
        src={`${window.BASE_PATH}/nothing-to-see-here.jpg`}
        alt="funny picture for the erased document"
        className="my-16 mx-auto"
      />
    </CardContainer>
  );
}
