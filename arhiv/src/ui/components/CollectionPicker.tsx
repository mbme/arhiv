import { useState } from 'react';
import { ensure } from 'utils';
import { DocumentId, DocumentType } from 'dto';
import { useSuspenseQuery } from 'utils/suspense';
import { DocumentPicker } from 'components/DocumentPicker';
import { Button, IconButton } from 'components/Button';
import { Ref } from 'components/Ref';

type Props = {
  ids: DocumentId[];
  onChange: (newIds: DocumentId[]) => void;
  collectionTypes: DocumentType[];
};
export function CollectionPicker({ collectionTypes, ids, onChange }: Props) {
  ensure(collectionTypes.length > 0, 'collectionTypes must not be empty');

  const [showPicker, setShowPicker] = useState(false);

  const { value, isUpdating } = useSuspenseQuery({
    typeName: 'GetDocuments',
    ids,
  });

  return (
    <>
      {showPicker && (
        <DocumentPicker
          documentTypes={collectionTypes}
          onSelected={({ id }) => {
            if (!ids.includes(id)) {
              onChange([...ids, id]);
            }
            setShowPicker(false);
          }}
          onCancel={() => {
            setShowPicker(false);
          }}
        />
      )}

      {value.documents.map((item) => (
        <div key={item.id} className="flex items-center gap-4">
          <Ref documentId={item.id} documentType={item.documentType} documentTitle={item.title} />

          <IconButton
            icon="x"
            size="sm"
            onClick={() => {
              onChange(ids.filter((id) => id !== item.id));
            }}
          />
        </div>
      ))}

      <Button
        variant="text"
        onClick={() => {
          setShowPicker(true);
        }}
        busy={isUpdating}
      >
        Pick {collectionTypes.join(', ')}...
      </Button>
    </>
  );
}
