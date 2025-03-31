import { useState } from 'react';
import { Dialog } from 'components/Dialog';
import { ExportedKey, ExportKeyForm } from './ExportKeyForm';
import { ExportKeyView } from './ExportKeyView';

interface Props {
  onCancel: () => void;
}
export function ExportKeyDialog({ onCancel }: Props) {
  const [exportedKey, setExportedKey] = useState<ExportedKey>();

  return (
    <Dialog onHide={onCancel} title="Export key">
      {exportedKey ? (
        <ExportKeyView exportedKey={exportedKey} />
      ) : (
        <ExportKeyForm onSuccess={setExportedKey} />
      )}
    </Dialog>
  );
}
