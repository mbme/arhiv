import { useEffect, useState } from 'react';
import { DocumentId } from 'dto';
import { ImageUploadDialog } from 'components/ImageUploadDialog';

type Props = {
  onSuccess: (documentId: DocumentId) => void;
};
export function ImagePasteHandler({ onSuccess }: Props) {
  const [file, setFile] = useState<File | null>(null);

  // TODO: handle drop events https://www.smashingmagazine.com/2018/01/drag-drop-file-uploader-vanilla-js/
  // TODO: upload multiple files
  useEffect(() => {
    const onPaste = (event: ClipboardEvent) => {
      for (const item of event.clipboardData?.items ?? []) {
        if (item.type.includes('image/')) {
          setFile(item.getAsFile());
          return;
        }
      }
    };

    document.addEventListener('paste', onPaste);

    return () => {
      document.removeEventListener('paste', onPaste);
    };
  }, []);

  if (!file) {
    return null;
  }

  return (
    <ImageUploadDialog
      file={file}
      onSuccess={(documentId) => {
        setFile(null);
        onSuccess(documentId);
      }}
      onCancel={() => {
        setFile(null);
      }}
    />
  );
}
