import { Dialog } from 'components/Dialog';
import { Catalog } from 'components/Catalog/Catalog';

type Props = {
  documentTypes: string[];
  onSelected: (documentId: string) => void;
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
