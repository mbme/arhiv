import { DocumentType, ERASED_DOCUMENT_TYPE } from 'dto';
import { cx } from 'utils';
import { getDocumentTypes, isErasedDocument } from 'utils/schema';
import { Badge } from 'components/Badge';
import { Button } from 'components/Button';

const ALL_DOCUMENT_TYPES = [
  ...getDocumentTypes(false), //
  ...getDocumentTypes(true),
];

const ALL_DOCUMENT_TYPES_EXCEPT_ERASED = ALL_DOCUMENT_TYPES.filter(
  (documentType) => !isErasedDocument(documentType),
);

type Props = {
  className?: string;
  selected: readonly DocumentType[];
  onChange: (documentTypes: DocumentType[]) => void;
};
export function DocumentTypeSettings({ className, selected, onChange }: Props) {
  const onClick = (documentType: DocumentType) => {
    if (selected.includes(documentType)) {
      onChange(selected.filter((item) => item !== documentType));
    } else {
      onChange([...selected, documentType]);
    }
  };

  const selectAll = () => {
    onChange([...ALL_DOCUMENT_TYPES_EXCEPT_ERASED]);
  };

  const selectNone = () => {
    onChange([]);
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
                checked={selected.includes(documentType)}
                onClick={() => onClick(documentType)}
              />
            );
          })}

          <Badge
            key={ERASED_DOCUMENT_TYPE}
            size="sm"
            className="line-through"
            label="ERASED"
            checked={selected.includes(ERASED_DOCUMENT_TYPE)}
            onClick={() => onClick(ERASED_DOCUMENT_TYPE)}
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
                checked={selected.includes(documentType)}
                onClick={() => onClick(documentType)}
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
    </div>
  );
}
