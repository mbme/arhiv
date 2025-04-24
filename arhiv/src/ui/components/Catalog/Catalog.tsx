import { useState } from 'react';
import { DocumentData, DocumentId, DocumentType } from 'dto';
import { cx } from 'utils';
import { getScaledImageUrl } from 'utils/network';
import { useSuspenseQuery } from 'utils/suspense';
import { useSelectionManager } from 'utils/selection-manager';
import { formatDocumentType } from 'utils/schema';
import { DateTime } from 'components/DateTime';
import { SearchInput } from 'components/SearchInput';
import { IconButton } from 'components/Button';
import { DocumentIcon } from 'components/DocumentIcon';
import { Pagination } from './Pagination';
import { CatalogFilter, type Filter } from './CatalogFilter';
import { CatalogItemBadges } from './CatalogItemBadges';

export type DocumentInfo = {
  id: DocumentId;
  documentType: DocumentType;
  data: DocumentData;
};

export type CatalogInfo = {
  query: string;
  page: number;
  documentTypes: DocumentType[];
};

type CatalogProps = {
  autofocus?: boolean;
  className?: string;
  filter: Filter;
  query: string;
  page: number;
  onQueryChange: (query: string) => void;
  onPageChange: (page: number) => void;
  onFilterChange: (filter: Filter) => void;
  onDocumentSelected: (info: DocumentInfo) => void;
  onCreateNote?: (title: string) => void;
  onConvertToCard?: (info: CatalogInfo) => void;
};

export function Catalog({
  autofocus = false,
  className,
  filter,
  query,
  page,
  onQueryChange,
  onPageChange,
  onFilterChange,
  onDocumentSelected,
  onCreateNote,
  onConvertToCard,
}: CatalogProps) {
  const [showSettings, setShowSettings] = useState(false);

  const { value: result, isUpdating } = useSuspenseQuery({
    typeName: 'ListDocuments',
    query,
    page,
    documentTypes: filter.documentTypes,
  });

  const { setRootEl } = useSelectionManager([result]);

  const items = result.documents.map((item) => (
    <div
      key={item.id}
      className={cx('cursor-pointer pr-2 py-2 sm-selectable hover:var-item-active-bg-color', {
        'bg-red-700/20': item.hasConflict,
      })}
      onClick={() => {
        onDocumentSelected({
          id: item.id,
          documentType: item.documentType,
          data: item.data,
        });
      }}
    >
      <div className="flex gap-3">
        <div className="shrink-0 w-[64px] h-[80px]">
          {item.cover ? (
            <img src={getScaledImageUrl(item.cover, 64, 80)} alt="cover" className="pl-2" />
          ) : (
            <DocumentIcon
              documentType={item.documentType}
              className="w-auto h-auto text-zinc-500"
            />
          )}
        </div>

        <div className="grow">
          <div className="flex justify-between">
            <div className="section-heading">{formatDocumentType(item.documentType)}</div>

            <DateTime
              className="font-mono text-sm shrink-0 text-gray-400"
              datetime={item.updatedAt}
              relative
            />
          </div>

          <div className="font-bold text-lg break-anywhere">{item.title}</div>

          <div className="empty:hidden mt-1 flex gap-2">
            <CatalogItemBadges documentType={item.documentType} data={item.data} />
          </div>
        </div>
      </div>
    </div>
  ));

  return (
    <div ref={setRootEl} className={className}>
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
        />

        <IconButton
          icon="cog"
          size="sm"
          onClick={() => {
            setShowSettings(!showSettings);
          }}
        />

        {onCreateNote && (
          <IconButton
            icon="add-document"
            size="sm"
            title="Create note"
            disabled={query.trim().length === 0}
            onClick={() => {
              onCreateNote(query.trim());
            }}
          />
        )}

        {onConvertToCard && (
          <IconButton
            icon="link-arrow"
            size="sm"
            title="Convert to card"
            disabled={query.trim().length === 0}
            onClick={() => {
              onConvertToCard({ query, page, documentTypes: filter.documentTypes });
            }}
          />
        )}
      </div>

      {showSettings && (
        <CatalogFilter
          className="mb-4 px-2 py-2 var-bg-tertiary-color"
          filter={filter}
          onChange={(newFilter) => {
            onFilterChange(newFilter);
            onPageChange(0);
          }}
        />
      )}

      <div className="divide-y border-gray-200">
        {items}
        {items.length === 0 && <div className="text-center mb-4">No results 😿</div>}
      </div>

      <Pagination page={page} hasMore={result.hasMore} onClick={onPageChange} />
    </div>
  );
}
