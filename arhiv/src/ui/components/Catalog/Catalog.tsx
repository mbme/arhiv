import { DocumentId, DocumentSubtype, DocumentType } from 'dto';
import { useSuspenseQuery } from 'utils/suspense';
import { DateTime } from 'components/DateTime';
import { SearchInput } from 'components/SearchInput';
import { IconButton } from 'components/Button';
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

  const items = result.documents.map((item) => (
    <div
      className="cursor-pointer px-2 py-3 transition-colors hover:bg-sky-100"
      key={item.id}
      onClick={() => onDocumentSelected(item.id, item.documentType, item.subtype)}
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter') {
          e.currentTarget.click();
        }
      }}
    >
      <div className="font-bold text-lg break-all">
        [{item.documentType || 'erased'}] {item.title}
      </div>

      <DateTime className="font-mono text-sm" datetime={item.updatedAt} relative />
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
          debounceMs={400}
        />

        <IconButton icon="cog" size="sm" onClick={() => onToggleSettings(!showSettings)} />
      </div>

      {showSettings && (
        <DocumentTypeSettings
          className="mb-4 px-2 py-2 bg-zinc-50"
          selectableTypes={documentTypes}
          selected={documentTypes}
          onChange={onIncludedDocumentTypesChange}
        />
      )}

      <div className="divide-y mb-6">
        {items}
        {items.length === 0 && <div className="text-center">No results 😿</div>}
      </div>

      <Pagination page={page} hasMore={result.hasMore} onClick={onPageChange} />
    </div>
  );
}
