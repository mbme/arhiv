import { startTransition, useState } from 'react';
import { NOTE_DOCUMENT_TYPE } from 'dto';
import { useKeydown, useSignal } from 'utils/hooks';
import { useAppController } from 'controller';
import { SuspenseCacheProvider } from 'components/SuspenseCacheProvider';
import { Button, IconButton } from 'components/Button';
import { DropdownMenu } from 'components/DropdownMenu';
import { DocumentPicker } from 'components/DocumentPicker';
import { FilePickerDialog } from 'components/FilePicker/FilePickerDialog';
import { NewDocumentDialog } from './NewDocumentDialog';
import { CommitOrSyncButton } from './CommitOrSyncButton';

export function WorkspaceHeader() {
  const app = useAppController();

  const [showNewDocumentDialog, setShowNewDocumentDialog] = useState(false);
  const [showFilePickerDialog, setShowFilePickerDialog] = useState(false);

  const [showSearchDialog, initialSearchQuery] = useSignal(app.workspace.$showSearchDialog);

  useKeydown(document.body, (e) => {
    // Search with Ctrl-K
    if (e.ctrlKey && e.code === 'KeyK' && !showSearchDialog) {
      e.preventDefault();
      app.workspace.showSearchDialog();
    }
  });

  return (
    <SuspenseCacheProvider cacheId="workspace-header">
      <nav className="fixed inset-x-0 top-0 z-20 var-bg-secondary-color pl-8 pr-4 flex flex-row gap-8">
        <Button variant="text" disabled>
          Player
        </Button>

        <IconButton
          icon="circle-half"
          title="Toggle light/dark theme"
          onClick={() => {
            app.toggleTheme();
          }}
          className="ml-auto"
        />

        <Button
          variant="text"
          leadingIcon="add-document"
          onClick={() => {
            setShowNewDocumentDialog(true);
          }}
        >
          <span className="hidden md:inline">New...</span>
        </Button>
        {showNewDocumentDialog && (
          <NewDocumentDialog
            onNewDocument={(documentType) => {
              app.workspace.newDocument(documentType);
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
          onClick={() => {
            startTransition(() => {
              app.workspace.showSearchDialog();
            });
          }}
        >
          <span className="hidden md:inline" title="Ctrl-K">
            Search
          </span>
        </Button>

        <CommitOrSyncButton />

        {showFilePickerDialog && (
          <FilePickerDialog
            onAttachmentCreated={(documentId) => {
              app.workspace.openDocument(documentId);
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
              app.workspace.hideSearchDialog();
              app.workspace.openDocument(info.id, true);
            }}
            onCancel={() => {
              app.workspace.hideSearchDialog();
            }}
            onCreateNote={(title) => {
              app.workspace.hideSearchDialog();
              app.workspace.newDocument(NOTE_DOCUMENT_TYPE, { title });
            }}
            initialQuery={initialSearchQuery}
          />
        )}

        <DropdownMenu
          align="bottom-right"
          options={[
            {
              text: 'Status',
              icon: 'info',
              onClick: () => {
                app.workspace.open({ variant: 'status' });
              },
            },

            {
              text: 'Catalog',
              icon: 'search-catalog',
              onClick: () => {
                app.workspace.open({ variant: 'catalog' });
              },
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
              onClick: () => {
                app.workspace.closeAll();
              },
            },
          ]}
        />
      </nav>
    </SuspenseCacheProvider>
  );
}
