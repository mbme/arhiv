import { useState } from 'react';
import { fuzzySearch } from 'utils';
import { getDocumentTypes, isErasedDocument } from 'utils/schema';
import { useSelectionManager } from 'utils/selection-manager';
import { DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';
import { SearchInput } from 'components/SearchInput';
import { IconVariant } from 'components/Icon';

type ActionType = 'Scrape URL' | 'Attach file';

const ACTIONS: ActionType[] = ['Attach file'];
if (window.FEATURES.scraper) {
  ACTIONS.unshift('Scrape URL');
}

function throwBadAction(value: never): never;
function throwBadAction(value: ActionType) {
  throw new Error(`Unknown ActionType: ${value}`);
}

const ACTION_ICONS: Record<ActionType, IconVariant> = {
  'Scrape URL': 'web',
  'Attach file': 'paperclip',
};

type Props = {
  onNewDocument: (documentType: DocumentType) => void;
  onScrape: () => void;
  onAttach: () => void;
  onCancel: () => void;
};

export function NewDocumentDialog({ onNewDocument, onScrape, onAttach, onCancel }: Props) {
  const [query, setQuery] = useState('');

  const { selectionManager, rootRef } = useSelectionManager([query]);

  const matchesQuery = (item: string) => fuzzySearch(query, item);

  const actions = ACTIONS.filter(matchesQuery);
  const documentTypes = getDocumentTypes(false).filter(matchesQuery);
  const collectionTypes = getDocumentTypes(true).filter(matchesQuery);

  const searchResultClass = 'justify-start capitalize sm-selectable';
  const headingClass = 'section-heading ml-4 mt-8 first:mt-0';

  const activateOnHover = (el: HTMLElement) => {
    selectionManager.activateElement(el);
  };

  return (
    <Dialog onHide={onCancel} title="Create new document">
      <SearchInput
        className="mb-8"
        autofocus
        initialValue=""
        placeholder="Filter actions"
        onSearch={setQuery}
        onKeyDown={(key) => {
          if (key === 'Escape') {
            onCancel();
            return true;
          }

          return selectionManager.handleKey(key);
        }}
      />

      <div
        ref={rootRef}
        className="flex flex-col gap-1 min-h-[20vh] md:max-h-[70vh] overflow-y-auto"
      >
        {actions.length > 0 && <h1 className={headingClass}>Actions</h1>}
        {actions.map((action) => (
          <Button
            key={action}
            variant="simple"
            leadingIcon={ACTION_ICONS[action]}
            onClick={() => {
              if (action === 'Scrape URL') {
                onScrape();
                return;
              }

              if (action === 'Attach file') {
                onAttach();
                return;
              }

              throwBadAction(action);
            }}
            onHover={activateOnHover}
            className={searchResultClass}
          >
            &nbsp; {action}
          </Button>
        ))}

        {documentTypes.length > 0 && <h1 className={headingClass}>Documents</h1>}
        {documentTypes.map((documentType) => {
          if (isErasedDocument(documentType)) {
            return null;
          }

          return (
            <Button
              key={documentType}
              variant="simple"
              onClick={() => onNewDocument(documentType)}
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
            onClick={() => onNewDocument(documentType)}
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
