import { useContext, useState } from 'react';
import { DocumentData, DocumentId, DocumentType, BLOBId } from 'dto';
import { isAttachment } from 'utils/schema';
import { isAudio, isImage } from 'utils';
import { getBlobUrl, getScaledImageUrl } from 'utils/network';
import { Button } from 'components/Button';
import { AudioPlayer } from 'components/AudioPlayer/AudioPlayer';
import { SuspenseImage } from 'components/SuspenseImage';
import { RefClickHandlerContext } from 'components/Ref';
import { Dialog } from 'components/Dialog';

type AttachmentPreviewBlockProps = {
  documentId: DocumentId;
  data: DocumentData;
  description?: string;
};
export function AttachmentPreviewBlock({
  documentId,
  data,
  description,
}: AttachmentPreviewBlockProps) {
  const refClickHandler = useContext(RefClickHandlerContext);

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

      <AttachmentPreview data={data} />
    </span>
  );
}

type AttachmentPreviewProps = {
  data: DocumentData;
};
export function AttachmentPreview({ data }: AttachmentPreviewProps) {
  const [showImageModal, setShowImageModal] = useState(false);

  const blobId = data['blob'] as BLOBId;
  const size = data['size'] as number;
  const mediaType = data['media_type'] as string;

  const blobUrl = getBlobUrl(blobId);

  if (isImage(mediaType)) {
    if (showImageModal) {
      return (
        <Dialog
          className="w-fit max-w-full"
          title={data['filename'] as string}
          onHide={() => {
            setShowImageModal(false);
          }}
        >
          <img className="max-w-full mx-auto" src={blobUrl} />
        </Dialog>
      );
    }

    const compressedImage = getScaledImageUrl(blobId, 600, 500);

    return (
      <SuspenseImage
        src={size < 1_000_000 ? blobUrl : compressedImage}
        alt=""
        className="max-h-96 mx-auto"
        onClick={() => {
          setShowImageModal(true);
        }}
      />
    );
  }

  if (isAudio(mediaType)) {
    return <AudioPlayer url={blobUrl} title="" artist="" />;
  }

  return null;
}

export function canPreview(documentType: DocumentType, data: DocumentData): boolean {
  if (!isAttachment(documentType)) {
    return false;
  }

  const mediaType = data['media_type'] as string;

  return isImage(mediaType) || isAudio(mediaType);
}
