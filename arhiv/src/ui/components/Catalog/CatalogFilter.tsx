import { DocumentType, ERASED_DOCUMENT_TYPE } from 'dto';
import { cx, withItems, withoutItems } from 'utils';
import { ALL_DOCUMENT_TYPES, getDocumentTypes, isErasedDocument } from 'utils/schema';
import { Badge } from 'components/Badge';
import { Button } from 'components/Button';
import { Checkbox } from 'components/Form/Checkbox';

export interface Filter {
  documentTypes: DocumentType[];
  onlyConflicts?: boolean;
}

const ALL_DOCUMENT_TYPES_EXCEPT_ERASED = withoutItems(ALL_DOCUMENT_TYPES, ERASED_DOCUMENT_TYPE);

type Props = {
  className?: string;
  filter: Filter;
  onChange: (filter: Filter) => void;
};
export function CatalogFilter({ className, filter, onChange }: Props) {
  const onClick = (documentType: DocumentType) => {
    if (filter.documentTypes.includes(documentType)) {
      onChange({
        ...filter,
        documentTypes: withoutItems(filter.documentTypes, documentType),
      });
    } else {
      onChange({
        ...filter,
        documentTypes: withItems(filter.documentTypes, documentType),
      });
    }
  };

  const selectAll = () => {
    onChange({
      ...filter,
      documentTypes: [...ALL_DOCUMENT_TYPES_EXCEPT_ERASED],
    });
  };

  const selectNone = () => {
    onChange({
      ...filter,
      documentTypes: [],
    });
  };

  const changeOnlyConflicts = (onlyConflicts: boolean) => {
    onChange({
      ...filter,
      onlyConflicts,
    });
  };

  return (
    <div className={cx('flex flex-wrap justify-between', className)}>
      <section className="w-1/2">
        <h1 className="section-heading ml-1 mb-2">Documents</h1>
        <div className="flex flex-wrap gap-x-2 gap-y-1">
          {getDocumentTypes(false).map((documentType) => {
            if (isErasedDocument(documentType)) {
              return null;
            }

            return (
              <Badge
                key={documentType}
                size="sm"
                label={documentType}
                checked={filter.documentTypes.includes(documentType)}
                onClick={() => {
                  onClick(documentType);
                }}
              />
            );
          })}

          <Badge
            key={ERASED_DOCUMENT_TYPE}
            size="sm"
            className="line-through"
            label="ERASED"
            checked={filter.documentTypes.includes(ERASED_DOCUMENT_TYPE)}
            onClick={() => {
              onClick(ERASED_DOCUMENT_TYPE);
            }}
          />
        </div>
      </section>

      <section className="w-1/2">
        <h1 className="section-heading ml-1 mb-2">Collections</h1>
        <div className="flex flex-wrap gap-x-2 gap-y-1">
          {getDocumentTypes(true).map((documentType) => {
            return (
              <Badge
                key={documentType}
                size="sm"
                label={documentType}
                checked={filter.documentTypes.includes(documentType)}
                onClick={() => {
                  onClick(documentType);
                }}
              />
            );
          })}
        </div>
      </section>

      <div className="flex justify-end gap-2 w-full mt-4 pr-2">
        <Button size="sm" variant="text" onClick={selectAll}>
          Select all
        </Button>
        /
        <Button size="sm" variant="text" onClick={selectNone}>
          Select none
        </Button>
      </div>

      <div className="flex gap-2 w-full pl-2">
        <label className="flex items-center gap-2">
          <Checkbox
            name="checkbox"
            value={filter.onlyConflicts ?? false}
            onChange={changeOnlyConflicts}
          />
          Only conflicts
        </label>
      </div>
    </div>
  );
}
