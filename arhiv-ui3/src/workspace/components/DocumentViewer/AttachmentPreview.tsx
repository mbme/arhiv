import { DocumentData } from '../../dto';
import { isImageAttachment } from '../../schema';

type AttachmentPreviewProps = {
  subtype: string;
  data: DocumentData;
};
export function AttachmentPreview({ subtype, data }: AttachmentPreviewProps) {
  const filename = data['filename'] as string;
  const blobId = data['blob'] as string;

  const blobUrl = `/blobs/${blobId}`;

  if (isImageAttachment(subtype)) {
    return <img src={blobUrl} alt={filename} className="max-h-96 mx-auto" />;
  }

  return null;
}
