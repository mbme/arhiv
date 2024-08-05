import { useState } from 'react';
import { fuzzySearch } from 'utils';
import { getDocumentTypes, isErasedDocument } from 'utils/schema';
import { useSelectionManager } from 'utils/selection-manager';
import { DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';
import { SearchInput } from 'components/SearchInput';
import { IconVariant } from 'components/Icon';

type ActionType = 'Attach file';

const ACTIONS: ActionType[] = ['Attach file'];

function throwBadAction(value: never): never;
function throwBadAction(value: ActionType) {
  throw new Error(`Unknown ActionType: ${value}`);
}

const ACTION_ICONS: Record<ActionType, IconVariant> = {
  'Attach file': 'paperclip',
};

type Props = {
  onNewDocument: (documentType: DocumentType) => void;
  onAttach: () => void;
  onCancel: () => void;
};

export function NewDocumentDialog({ onNewDocument, onAttach, onCancel }: Props) {
  const [query, setQuery] = useState('');

  const { selectionManager, setRootEl } = useSelectionManager([query]);

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
    <Dialog innerRef={setRootEl} onHide={onCancel} title="Create new document">
      <SearchInput
        className="mb-8"
        autofocus
        initialValue=""
        placeholder="Filter actions"
        onSearch={setQuery}
      />

      <div className="flex flex-col gap-1 min-h-[20vh] overflow-y-auto">
        {actions.length > 0 && <h1 className={headingClass}>Actions</h1>}
        {actions.map((action) => (
          <Button
            key={action}
            variant="simple"
            leadingIcon={ACTION_ICONS[action]}
            onClick={() => {
              // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
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
