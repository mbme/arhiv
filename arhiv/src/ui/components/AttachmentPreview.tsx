import { useContext } from 'react';
import { DocumentData, DocumentId, DocumentType, DocumentSubtype } from 'dto';
import { isAttachment, isAudioAttachment, isImageAttachment } from 'utils/schema';
import { Button } from 'components/Button';
import { AudioPlayer } from 'components/AudioPlayer/AudioPlayer';
import { SuspenseImage } from 'components/SuspenseImage';
import { RefClickHandlerContext } from 'components/Ref';

type AttachmentPreviewProps = {
  documentId: DocumentId;
  subtype: DocumentSubtype;
  data: DocumentData;
  description?: string;
};
export function AttachmentPreview({
  documentId,
  subtype,
  data,
  description,
}: AttachmentPreviewProps) {
  const refClickHandler = useContext(RefClickHandlerContext);

  const preview = getAttachmentPreview(subtype, data);

  if (!preview) {
    throw new Error(`Can't preview ${subtype} attachment ${documentId}`);
  }

  return (
    <span className="block w-full group">
      <span className="flex space-between items-center">
        <span className="text-blue-900 pointer font-serif pl-1">{description}</span>

        <Button
          variant="text"
          onClick={() => {
            refClickHandler(documentId);
          }}
          className="ml-auto text-sm  transition invisible opacity-0 group-hover:visible group-hover:opacity-100"
          trailingIcon="link-arrow"
          size="sm"
        >
          open
        </Button>
      </span>
      {preview}
    </span>
  );
}

export function canPreview(documentType: DocumentType, subtype: DocumentSubtype): boolean {
  if (!isAttachment(documentType)) {
    return false;
  }

  return isImageAttachment(subtype) || isAudioAttachment(subtype);
}

export function getAttachmentPreview(subtype: DocumentSubtype, data: DocumentData) {
  const blobId = data['blob'] as string;
  const size = data['size'] as number;
  const blobUrl = `${window.BASE_PATH}/blobs/${blobId}`;

  if (isImageAttachment(subtype)) {
    const compressedImage = `${window.BASE_PATH}/blobs/images/${blobId}?max_w=600&max_h=500`;

    return (
      <SuspenseImage
        src={size < 1_000_000 ? blobUrl : compressedImage}
        alt=""
        className="max-h-96 mx-auto"
      />
    );
  }

  if (isAudioAttachment(subtype)) {
    return <AudioPlayer url={blobUrl} title="" artist="" />;
  }

  return null;
}
