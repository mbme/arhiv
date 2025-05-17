import { startTransition } from 'react';
import { ASSET_DOCUMENT_TYPE, ERASED_DOCUMENT_TYPE, NOTE_DOCUMENT_TYPE } from 'dto';
import { ALL_DOCUMENT_TYPES } from 'utils/schema';
import { withoutItems } from 'utils';
import { useKeydown, useSignal } from 'utils/hooks';
import { RPC } from 'utils/network';
import { useAppController } from 'controller';
import { SuspenseCacheProvider } from 'components/SuspenseCacheProvider';
import { Button } from 'components/Button';
import { DropdownMenu } from 'components/DropdownMenu';
import { DocumentPicker } from 'components/DocumentPicker';
import { NewDocumentDialog } from './NewDocumentDialog';
import { CommitButton } from './CommitButton';
import { ConflictsButton } from './ConflictsButton';
import { ExportKeyDialog } from './ExportKeyDialog';
import { OutdatedChecker } from './OutdatedChecker';

const SEARCH_DOCUMENT_TYPES = withoutItems(
  ALL_DOCUMENT_TYPES,
  ERASED_DOCUMENT_TYPE,
  ASSET_DOCUMENT_TYPE,
);

export function WorkspaceHeader() {
  const app = useAppController();

  const [showSearchDialog, initialSearchQuery] = useSignal(app.workspace.$showSearchDialog);
  const showNewDocumentDialog = useSignal(app.workspace.$showNewDocumentDialog);
  const showExportKeyDialog = useSignal(app.workspace.$showExportKeyDialog);

  useKeydown(document.body, (e) => {
    // Search with Ctrl-K
    if (e.ctrlKey && e.code === 'KeyK' && !showSearchDialog) {
      e.preventDefault();
      startTransition(() => {
        app.workspace.showSearchDialog();
      });
    }

    // Create new document with Ctrl-N
    if (e.ctrlKey && e.code === 'KeyN' && !showNewDocumentDialog) {
      e.preventDefault();
      app.workspace.showNewDocumentDialog();
    }
  });

  return (
    <SuspenseCacheProvider cacheId="workspace-header">
      <nav className="fixed inset-x-0 top-0 z-20 var-bg-secondary-color xs:px-4 flex flex-row gap-4 xs:gap-8">
        <OutdatedChecker />

        <div className="mr-auto" />

        <ConflictsButton
          onClick={() => {
            app.workspace.open({ variant: 'catalog', onlyConflicts: true });
          }}
        />

        <Button
          variant="text"
          leadingIcon="add-document"
          onClick={() => {
            app.workspace.showNewDocumentDialog();
          }}
        >
          <span className="hidden md:inline" title="Ctrl-N">
            New...
          </span>
        </Button>
        {showNewDocumentDialog && (
          <NewDocumentDialog
            onNewDocument={(documentType) => {
              app.workspace.hideNewDocumentDialog();
              app.workspace.newDocument(documentType);
            }}
            onAssetCreated={(assets) => {
              app.workspace.hideNewDocumentDialog();

              if (assets.length === 1) {
                app.workspace.openDocument(assets[0]!);
              } else {
                app.workspace.openDocumentsList('File upload results', assets);
              }
            }}
            onCancel={() => {
              app.workspace.hideNewDocumentDialog();
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

        <CommitButton />

        {showSearchDialog && (
          <DocumentPicker
            title="Search"
            documentTypes={SEARCH_DOCUMENT_TYPES}
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
            onConvertToCard={({ query, page, documentTypes }) => {
              app.workspace.hideSearchDialog();
              app.workspace.open({ variant: 'catalog', query, page, documentTypes });
            }}
            initialQuery={initialSearchQuery}
          />
        )}

        {showExportKeyDialog && (
          <ExportKeyDialog
            onCancel={() => {
              app.workspace.hideExportKeyDialog();
            }}
          />
        )}

        <DropdownMenu
          align="bottom-right"
          options={[
            {
              text: 'Reload',
              icon: 'refresh',
              onClick: () => {
                app.workspace.reload();
              },
            },

            {
              text: 'Toggle light/dark theme',
              icon: 'circle-half',

              onClick: () => {
                app.toggleTheme();
              },
            },

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

            {
              text: 'Export key',
              icon: 'key',
              onClick: () => {
                app.workspace.showExportKeyDialog();
              },
            },

            {
              text: 'Lock arhiv',
              icon: 'lock',
              onClick: () => {
                RPC.LockArhiv({}).then(
                  () => {
                    location.reload();
                  },
                  (err: unknown) => {
                    console.error('Failed to lock Arhiv:', err);
                  },
                );
              },
            },
          ]}
        />
      </nav>
    </SuspenseCacheProvider>
  );
}
