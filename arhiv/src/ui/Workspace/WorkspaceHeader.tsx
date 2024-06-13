import { startTransition, useState } from 'react';
import { NOTE_DOCUMENT_TYPE } from 'dto';
import { useKeydown } from 'utils/hooks';
import { SuspenseCacheProvider } from 'components/SuspenseCacheProvider';
import { Button } from 'components/Button';
import { DropdownMenu } from 'components/DropdownMenu';
import { ScraperDialog } from 'components/ScraperDialog';
import { DocumentPicker } from 'components/DocumentPicker';
import { FilePickerDialog } from 'components/FilePicker/FilePickerDialog';
import { WorkspaceController } from './workspace-reducer';
import { NewDocumentDialog } from './NewDocumentDialog';
import { ImagePasteHandler } from './ImagePasteHandler';
import { CommitOrSyncButton } from './CommitOrSyncButton';

type Props = {
  controller: WorkspaceController;
};
export function WorkspaceHeader({ controller }: Props) {
  const [showNewDocumentDialog, setShowNewDocumentDialog] = useState(false);
  const [showScraperDialog, setShowScraperDialog] = useState(false);
  const [showFilePickerDialog, setShowFilePickerDialog] = useState(false);
  const [showSearchDialog, setShowSearchDialog] = useState(false);

  useKeydown(document.body, (e) => {
    // Search with Ctrl-K
    if (e.ctrlKey && e.code === 'KeyK' && !showSearchDialog) {
      e.preventDefault();
      startTransition(() => {
        setShowSearchDialog(true);
      });
    }
  });

  return (
    <SuspenseCacheProvider cacheId="workspace-header">
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
              controller.open({ variant: 'new-document', documentType });
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
          onClick={() => startTransition(() => setShowSearchDialog(true))}
        >
          <span className="hidden md:inline" title="Ctrl-K">
            Search
          </span>
        </Button>

        <CommitOrSyncButton />

        {showScraperDialog && (
          <ScraperDialog
            onSuccess={(url, ids) => {
              controller.open({ variant: 'scrape-result', url, ids });
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
              controller.openDocument(documentId);
              setShowFilePickerDialog(false);
            }}
            onCancel={() => {
              setShowFilePickerDialog(false);
            }}
          />
        )}

        {showSearchDialog && (
          <DocumentPicker
            title="Search"
            hideOnSelect
            onSelected={(info) => {
              setShowSearchDialog(false);
              controller.openDocument(info.id, true);
            }}
            onCancel={() => setShowSearchDialog(false)}
            onCreateNote={(title) => {
              setShowSearchDialog(false);
              controller.open({
                variant: 'new-document',
                documentType: NOTE_DOCUMENT_TYPE,
                data: { title },
              });
            }}
          />
        )}

        <ImagePasteHandler
          onSuccess={(documentId) => {
            controller.openDocument(documentId);
          }}
        />

        <DropdownMenu
          align="bottom-right"
          options={[
            {
              text: 'Status',
              icon: 'info',
              onClick: () => controller.open({ variant: 'status' }),
            },

            {
              text: 'Catalog',
              icon: 'search-catalog',
              onClick: () => controller.open({ variant: 'catalog' }),
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
              onClick: () => controller.closeAll(),
            },
          ]}
        />
      </nav>
    </SuspenseCacheProvider>
  );
}
