import { getDocumentTypes, isErasedDocument } from 'utils/schema';
import { DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';

type Props = {
  onNewDocument: (documentType: DocumentType) => void;
  onCancel: () => void;
};
export function NewDocumentDialog({ onNewDocument, onCancel }: Props) {
  const documentTypes = getDocumentTypes(false);
  const collectionTypes = getDocumentTypes(true);

  return (
    <Dialog onHide={onCancel} title="Create new document">
      <div className="flex justify-around my-8">
        <section>
          <h1 className="section-heading ml-4">Documents</h1>
          {documentTypes.map((documentType) => {
            if (isErasedDocument(documentType)) {
              return null;
            }

            return (
              <Button
                key={documentType}
                variant="simple"
                onClick={() => onNewDocument(documentType)}
              >
                {documentType}
              </Button>
            );
          })}
        </section>

        <section>
          <h1 className="section-heading ml-4">Collections</h1>
          {collectionTypes.map((documentType) => (
            <Button key={documentType} variant="simple" onClick={() => onNewDocument(documentType)}>
              {documentType}
            </Button>
          ))}
        </section>
      </div>
    </Dialog>
  );
}
