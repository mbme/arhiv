import { DocumentId, DocumentType } from 'dto';
import { Dialog } from 'components/Dialog';
import { Catalog } from 'components/Catalog/Catalog';

type Props = {
  documentTypes: DocumentType[];
  onSelected: (id: DocumentId) => void;
  onCancel: () => void;
};

export function DocumentPicker({ documentTypes, onSelected, onCancel }: Props) {
  return (
    <Dialog title={`Pick ${documentTypes.join(', ')}`} onHide={onCancel}>
      <div className="px-2">
        <Catalog autofocus documentTypes={documentTypes} onDocumentSelected={onSelected} />
      </div>
    </Dialog>
  );
}
