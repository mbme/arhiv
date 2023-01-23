import { useState } from 'preact/hooks';
import { Ref, useDocuments } from 'components/Ref';
import { DocumentPicker } from 'components/DocumentPicker';
import { Button, IconButton } from 'components/Button';
import { QueryError } from 'components/QueryError';

type Props = {
  documentType: string;
  ids: string[];
  multiple: boolean;
  readonly: boolean;
  disabled: boolean;
  onChange: (ids: string[]) => void;
  onRefClick: (documentId: string) => void;
};

export function RefInput({
  documentType,
  ids,
  multiple,
  readonly,
  disabled,
  onChange,
  onRefClick,
}: Props) {
  const [showPicker, setShowPicker] = useState(false);

  const canAdd = ids.length === 0 || multiple;

  const { documents, error, inProgress } = useDocuments(ids);

  if (error) {
    return <QueryError error={error} />;
  }

  if (inProgress || !documents) {
    return null;
  }

  return (
    <>
      {showPicker && (
        <DocumentPicker
          documentType={documentType}
          onSelected={(documentId) => {
            if (!ids.includes(documentId)) {
              onChange([...ids, documentId]);
            }
            setShowPicker(false);
          }}
          onCancel={() => setShowPicker(false)}
        />
      )}

      {documents.map((item) => (
        <div className="flex items-center gap-4" key={item.id}>
          <Ref
            documentId={item.id}
            documentType={item.documentType}
            subtype={item.subtype}
            documentTitle={item.title}
            onClick={() => onRefClick(item.id)}
          />

          {!readonly && !disabled && (
            <IconButton
              icon="x"
              size="sm"
              onClick={() => onChange(ids.filter((id) => id !== item.id))}
            />
          )}
        </div>
      ))}

      {canAdd && (
        <Button variant="text" onClick={() => setShowPicker(true)} disabled={readonly || disabled}>
          Pick {documentType}...
        </Button>
      )}
    </>
  );
}
