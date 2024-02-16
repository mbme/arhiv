import { useState } from 'react';
import { Button } from 'components/Button';
import { DropdownMenu } from 'components/DropdownMenu';
import { ScraperDialog } from 'components/ScraperDialog';
import { FilePickerDialog } from 'components/FilePicker/FilePickerDialog';
import { WorkspaceDispatch, useWorkspaceActions } from './workspace-reducer';
import { NewDocumentDialog } from './NewDocumentDialog';
import { ImagePasteHandler } from './ImagePasteHandler';
import { CommitOrSyncButton } from './CommitOrSyncButton';

type Props = {
  dispatch: WorkspaceDispatch;
};
export function WorkspaceHeader({ dispatch }: Props) {
  const { open, closeAll } = useWorkspaceActions(dispatch);

  const [showNewDocumentDialog, setShowNewDocumentDialog] = useState(false);
  const [showScraperDialog, setShowScraperDialog] = useState(false);
  const [showFilePickerDialog, setShowFilePickerDialog] = useState(false);

  return (
    <nav className="fixed inset-x-0 top-0 z-20 bg-zinc-200 var-bg-color pl-8 pr-4 flex flex-row gap-8">
      <Button variant="text" disabled>
        Player
      </Button>

      <Button
        variant="text"
        leadingIcon="add-document"
        onClick={() => setShowNewDocumentDialog(true)}
        className="ml-auto"
      >
        <span className="hidden md:inline">New...</span>
      </Button>
      {showNewDocumentDialog && (
        <NewDocumentDialog
          onNewDocument={(documentType) => {
            open({ variant: 'new-document', documentType });
            setShowNewDocumentDialog(false);
          }}
          onScrape={() => {
            setShowScraperDialog(true);
            setShowNewDocumentDialog(false);
          }}
          onAttach={() => {
            setShowFilePickerDialog(true);
            setShowNewDocumentDialog(false);
          }}
          onCancel={() => {
            setShowNewDocumentDialog(false);
          }}
        />
      )}

      <Button
        variant="text"
        leadingIcon="search-catalog"
        onClick={() => open({ variant: 'catalog' })}
      >
        <span className="hidden md:inline">Search</span>
      </Button>

      <CommitOrSyncButton />

      {showScraperDialog && (
        <ScraperDialog
          onSuccess={(url, ids) => {
            open({ variant: 'scrape-result', url, ids });
            setShowScraperDialog(false);
          }}
          onCancel={() => {
            setShowScraperDialog(false);
          }}
        />
      )}

      {showFilePickerDialog && (
        <FilePickerDialog
          onAttachmentCreated={(documentId) => {
            open({ variant: 'document', documentId });
            setShowFilePickerDialog(false);
          }}
          onCancel={() => {
            setShowFilePickerDialog(false);
          }}
        />
      )}

      <ImagePasteHandler
        onSuccess={(documentId) => {
          open({ variant: 'document', documentId });
        }}
      />

      <DropdownMenu
        align="bottom-right"
        options={[
          {
            text: 'Status',
            icon: 'info',
            onClick: () => open({ variant: 'status' }),
          },

          process.env.NODE_ENV === 'development' && {
            text: 'Components Demo',
            onClick: () => {
              window.location.search = 'DEMO';
            },
          },

          {
            text: 'Close cards',
            icon: 'x',
            onClick: closeAll,
          },
        ]}
      />
    </nav>
  );
}
