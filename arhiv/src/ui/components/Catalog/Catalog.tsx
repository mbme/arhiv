import { DocumentId, DocumentSubtype, DocumentType } from 'dto';
import { getScaledImageUrl } from 'utils';
import { useSuspenseQuery } from 'utils/suspense';
import { useSelectionManager } from 'utils/selection-manager';
import { formatDocumentType } from 'utils/schema';
import { DateTime } from 'components/DateTime';
import { SearchInput } from 'components/SearchInput';
import { IconButton } from 'components/Button';
import { DocumentIcon } from 'components/DocumentIcon';
import { Pagination } from './Pagination';
import { DocumentTypeSettings } from './DocumentTypeSettings';

type CatalogProps = {
  autofocus?: boolean;
  className?: string;
  documentTypes: DocumentType[];
  query: string;
  page: number;
  showSettings: boolean;
  onQueryChange: (query: string) => void;
  onPageChange: (page: number) => void;
  onToggleSettings: (showSettings: boolean) => void;
  onIncludedDocumentTypesChange: (documentTypes: DocumentType[]) => void;
  onDocumentSelected: (
    id: DocumentId,
    documentType: DocumentType,
    subtype: DocumentSubtype,
  ) => void;
};

export function Catalog({
  autofocus = false,
  className,
  documentTypes,
  query,
  page,
  showSettings,
  onQueryChange,
  onPageChange,
  onToggleSettings,
  onIncludedDocumentTypesChange,
  onDocumentSelected,
}: CatalogProps) {
  const { value: result, isUpdating } = useSuspenseQuery({
    typeName: 'ListDocuments',
    query,
    page,
    documentTypes,
  });

  const { selectionManager, rootRef } = useSelectionManager([result]);

  const items = result.documents.map((item) => (
    <div
      key={item.id}
      className="cursor-pointer pr-2 py-3 sm-selectable hover:bg-sky-100"
      onClick={() => onDocumentSelected(item.id, item.documentType, item.subtype)}
    >
      <div className="flex gap-3">
        <div className="shrink-0 w-[64px] h-[80px]">
          {item.cover ? (
            <img src={getScaledImageUrl(item.cover, 64, 80)} alt="cover" className="pl-2" />
          ) : (
            <DocumentIcon documentType={item.documentType} />
          )}
        </div>

        <div className="grow">
          <div className="flex justify-between">
            <div className="section-heading">
              {formatDocumentType(item.documentType, item.subtype)}
            </div>

            <DateTime
              className="font-mono text-sm shrink-0 text-gray-400"
              datetime={item.updatedAt}
              relative
            />
          </div>

          <div className="font-bold text-lg break-anywhere">{item.title}</div>
        </div>
      </div>
    </div>
  ));

  return (
    <div className={className}>
      <div className="flex gap-4 items-center mb-4">
        <SearchInput
          className="flex-auto"
          initialValue={query}
          onSearch={(newQuery) => {
            onQueryChange(newQuery);
            onPageChange(0);
          }}
          busy={isUpdating}
          autofocus={autofocus}
          debounceMs={700}
          onKeyDown={(key) => {
            return selectionManager.handleKey(key);
          }}
        />

        <IconButton icon="cog" size="sm" onClick={() => onToggleSettings(!showSettings)} />
      </div>

      {showSettings && (
        <DocumentTypeSettings
          className="mb-4 px-2 py-2 bg-zinc-50"
          selected={documentTypes}
          onChange={(newDocumentTypes) => {
            onIncludedDocumentTypesChange(newDocumentTypes);
            onPageChange(0);
          }}
        />
      )}

      <div ref={rootRef} className="divide-y">
        {items}
        {items.length === 0 && <div className="text-center">No results 😿</div>}
      </div>

      <Pagination page={page} hasMore={result.hasMore} onClick={onPageChange} />
    </div>
  );
}
