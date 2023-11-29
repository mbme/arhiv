import { DocumentDTO } from 'dto';
import { copyTextToClipbard, getDocumentUrl } from 'utils';
import { DropdownMenu } from 'components/DropdownMenu';
import { CardContainer } from 'Workspace/CardContainer';
import { ProgressLocker } from 'components/ProgressLocker';
import { DocumentViewerHead } from '../DocumentEditor/DocumentViewerHead';

type Props = {
  document: DocumentDTO;
  isUpdating: boolean;
};

export function ErasedDocumentCard({ document, isUpdating }: Props) {
  return (
    <CardContainer
      leftToolbar={
        <DropdownMenu
          icon="dots-horizontal"
          align="bottom-left"
          options={[
            {
              text: `ID ${document.id}`,
              icon: 'clipboard',
              onClick: () => {
                void copyTextToClipbard(document.id);
              },
            },
            {
              text: 'Copy link',
              icon: 'clipboard',
              onClick: () => {
                void copyTextToClipbard(getDocumentUrl(document.id));
              },
            },
          ]}
        />
      }
    >
      {isUpdating && <ProgressLocker />}

      <DocumentViewerHead
        documentType={document.documentType}
        subtype={document.subtype}
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
