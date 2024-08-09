import { useState } from 'react';
import { fuzzySearch } from 'utils';
import { getDocumentTypes, isErasedDocument } from 'utils/schema';
import { useSelectionManager } from 'utils/selection-manager';
import { DocumentId, DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';
import { SearchInput } from 'components/SearchInput';
import { FilePickerDialog } from 'components/FilePicker/FilePickerDialog';

type Props = {
  onNewDocument: (documentType: DocumentType) => void;
  onAttach: (attachmentId: DocumentId) => void;
  onCancel: () => void;
};

export function NewDocumentDialog({ onNewDocument, onAttach, onCancel }: Props) {
  const [query, setQuery] = useState('');

  const [showFilePickerDialog, setShowFilePickerDialog] = useState(false);

  const { selectionManager, setRootEl } = useSelectionManager([query]);

  const matchesQuery = (item: string) => fuzzySearch(query, item);

  const documentTypes = getDocumentTypes(false).filter(matchesQuery);
  const collectionTypes = getDocumentTypes(true).filter(matchesQuery);

  const searchResultClass = 'justify-start capitalize sm-selectable';
  const headingClass = 'section-heading ml-4 mt-8 first:mt-0';

  const activateOnHover = (el: HTMLElement) => {
    selectionManager.activateElement(el);
  };

  if (showFilePickerDialog) {
    return (
      <FilePickerDialog
        onAttachmentCreated={(documentId) => {
          onAttach(documentId);
          setShowFilePickerDialog(false);
        }}
        onCancel={() => {
          onCancel();
        }}
      />
    );
  }

  return (
    <Dialog innerRef={setRootEl} onHide={onCancel} title="Create new document">
      <SearchInput
        className="mb-8"
        autofocus
        initialValue=""
        placeholder="Filter actions"
        onSearch={setQuery}
      />

      <div className="flex flex-col gap-1 min-h-[20vh] overflow-y-auto">
        <h1 className={headingClass}>Actions</h1>

        <Button
          variant="simple"
          leadingIcon="paperclip"
          onClick={() => {
            setShowFilePickerDialog(true);
          }}
          onHover={activateOnHover}
          className={searchResultClass}
        >
          &nbsp; Attach file
        </Button>

        {documentTypes.length > 0 && <h1 className={headingClass}>Documents</h1>}
        {documentTypes.map((documentType) => {
          if (isErasedDocument(documentType)) {
            return null;
          }

          return (
            <Button
              key={documentType}
              variant="simple"
              onClick={() => {
                onNewDocument(documentType);
              }}
              onHover={activateOnHover}
              className={searchResultClass}
            >
              {documentType}
            </Button>
          );
        })}

        {collectionTypes.length > 0 && <h1 className={headingClass}>Collections</h1>}
        {collectionTypes.map((documentType) => (
          <Button
            key={documentType}
            variant="simple"
            onClick={() => {
              onNewDocument(documentType);
            }}
            onHover={activateOnHover}
            className={searchResultClass}
          >
            {documentType}
          </Button>
        ))}
      </div>
    </Dialog>
  );
}
