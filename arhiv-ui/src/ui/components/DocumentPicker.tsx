import { Dialog } from 'components/Dialog';
import { Catalog } from 'components/Catalog/Catalog';

type Props = {
  documentType: string;
  onSelected: (documentId: string) => void;
  onCancel: () => void;
};

export function DocumentPicker({ documentType, onSelected, onCancel }: Props) {
  return (
    <Dialog title={`Pick ${documentType}`} onHide={onCancel}>
      <div className="px-2">
        <Catalog autofocus documentType={documentType} onDocumentSelected={onSelected} />
      </div>
    </Dialog>
  );
}
