import { useEffect, useRef, useState } from 'react';
import { ArrayElement, fuzzySearch } from 'utils';
import { getDocumentTypes, isErasedDocument } from 'utils/schema';
import { DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';
import { SearchInput } from 'components/SearchInput';
import { IconVariant } from 'components/Icon';

const getItems = (root: HTMLElement) => root.querySelectorAll<HTMLElement>('.is-search-result');

const activateItem = (root: HTMLElement, index: number) => {
  getItems(root).forEach((el, i) => {
    const isActive = index === i;
    el.dataset['selected'] = isActive ? 'true' : '';

    if (isActive) {
      el.scrollIntoView({ block: 'nearest' });
    }
  });
};

const activateNextItem = (root: HTMLElement, increment = true) => {
  const items = Array.from(getItems(root));

  if (items.length === 0) {
    return;
  }

  const index = items.findIndex((el) => el.dataset['selected'] === 'true');

  let newIndex = index + (increment ? 1 : -1);
  if (newIndex >= items.length) {
    newIndex = 0;
  }
  if (newIndex < 0) {
    newIndex = items.length - 1;
  }

  activateItem(root, newIndex);
};

const activateElement = (root: HTMLElement, el: HTMLElement) => {
  const items = Array.from(getItems(root));
  const index = items.indexOf(el);

  activateItem(root, index);
};

const clickActive = (root: HTMLElement) => {
  const items = Array.from(getItems(root));

  const activeItem = items.find((el) => el.dataset['selected'] === 'true');

  activeItem?.click();
};

const ACTIONS = ['Scrape URL', 'Attach file'] as const;
type ActionType = ArrayElement<typeof ACTIONS>;

function throwBadAction(value: never): never;
function throwBadAction(value: ActionType) {
  throw new Error(`Unknown CardVariant: ${value}`);
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
  const resultsRef = useRef<HTMLDivElement>(null);

  const [query, setQuery] = useState('');

  useEffect(() => {
    if (!resultsRef.current) {
      return;
    }

    activateItem(resultsRef.current, 0);
  }, [query]);

  const matchesQuery = (item: string) => fuzzySearch(query, item);

  const actions = ACTIONS.filter(matchesQuery);
  const documentTypes = getDocumentTypes(false).filter(matchesQuery);
  const collectionTypes = getDocumentTypes(true).filter(matchesQuery);

  const searchResultClass = `justify-start capitalize is-search-result data-[selected=true]:var-item-active-bg-color`;
  const headingClass = 'section-heading ml-4 mt-8 first:mt-0';

  const activateOnHover = (el: HTMLElement) => {
    const root = resultsRef.current;
    if (!root) {
      throw new Error('root element is missing');
    }

    activateElement(root, el);
  };

  return (
    <Dialog onHide={onCancel} title="Create new document">
      <div className="modal-content">
        <SearchInput
          className="mb-8"
          autofocus
          initialValue=""
          placeholder="Filter actions"
          onSearch={setQuery}
          onKeyDown={(key) => {
            const root = resultsRef.current;
            if (!root) {
              throw new Error('root element is missing');
            }

            switch (key) {
              case 'Enter': {
                clickActive(root);

                return true;
              }
              case 'ArrowUp': {
                activateNextItem(root, false);

                return true;
              }
              case 'ArrowDown': {
                activateNextItem(root, true);

                return true;
              }
              case 'Escape': {
                onCancel();
                return true;
              }
              default: {
                return false;
              }
            }
          }}
        />

        <div
          ref={resultsRef}
          className="flex flex-col gap-1 min-h-[20vh] max-h-[70vh] overflow-y-auto"
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
      </div>
    </Dialog>
  );
}
