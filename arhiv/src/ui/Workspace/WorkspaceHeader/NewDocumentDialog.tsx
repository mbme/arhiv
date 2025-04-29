import { useState } from 'react';
import { Callback, fuzzySearch } from 'utils';
import { getDocumentTypes, isErasedDocument } from 'utils/schema';
import { useSelectionManager } from 'utils/selection-manager';
import { DocumentId, DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';
import { IconVariant } from 'components/Icon';
import { SearchInput } from 'components/SearchInput';
import { FilePickerDialog } from 'components/FilePicker/FilePickerDialog';
import { FileUploadDialog } from 'components/FileUploadDialog';

type Item = {
  name: string;
  leadingIcon?: IconVariant;
  onClick: Callback;
};

type ResultsSectionProps = {
  heading: string;
  filter: string;
  items: Item[];
};
function ResultsSection({ heading, filter, items }: ResultsSectionProps) {
  const visibleItems = items.filter((item) => fuzzySearch(filter, item.name));

  if (visibleItems.length === 0) {
    return null;
  }

  return (
    <>
      <h1 className="section-heading ml-4 mt-8 first:mt-0">{heading}</h1>

      {visibleItems.map((item) => (
        <Button
          key={item.name}
          variant="simple"
          leadingIcon={item.leadingIcon}
          onClick={item.onClick}
          className="justify-start capitalize sm-selectable"
        >
          {item.leadingIcon && <>&nbsp;</>}
          {item.name}
        </Button>
      ))}
    </>
  );
}

type Props = {
  onNewDocument: (documentType: DocumentType) => void;
  onAssetCreated: (assetId: DocumentId) => void;
  onCancel: () => void;
};

export function NewDocumentDialog({ onNewDocument, onAssetCreated, onCancel }: Props) {
  const [filter, setFilter] = useState('');

  const [showFilePickerDialog, setShowFilePickerDialog] = useState(false);
  const [showFileUploadDialog, setShowFileUploadDialog] = useState(false);

  const { setRootEl } = useSelectionManager([filter]);

  const documentTypes = getDocumentTypes(false).filter(
    (documentType) => !isErasedDocument(documentType),
  );
  const collectionTypes = getDocumentTypes(true);

  if (showFilePickerDialog) {
    return (
      <FilePickerDialog
        onAssetCreated={(documentId) => {
          onAssetCreated(documentId);
          setShowFilePickerDialog(false);
        }}
        onCancel={() => {
          onCancel();
        }}
      />
    );
  }

  if (showFileUploadDialog) {
    return (
      <FileUploadDialog
        onAssetCreated={(documentId) => {
          onAssetCreated(documentId);
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
        onSearch={setFilter}
      />

      <div className="flex flex-col gap-1 min-h-[20vh] overflow-y-auto">
        <ResultsSection
          heading="Actions"
          filter={filter}
          items={[
            {
              name: 'Create asset',
              leadingIcon: 'paperclip',
              onClick: () => {
                setShowFilePickerDialog(true);
              },
            },
            {
              name: 'Upload file',
              leadingIcon: 'upload',
              onClick: () => {
                setShowFileUploadDialog(true);
              },
            },
          ]}
        />

        <ResultsSection
          heading="Documents"
          filter={filter}
          items={documentTypes.map((documentType) => ({
            name: documentType,
            onClick() {
              onNewDocument(documentType);
            },
          }))}
        />

        <ResultsSection
          heading="Collections"
          filter={filter}
          items={collectionTypes.map((documentType) => ({
            name: documentType,
            onClick() {
              onNewDocument(documentType);
            },
          }))}
        />
      </div>
    </Dialog>
  );
}
