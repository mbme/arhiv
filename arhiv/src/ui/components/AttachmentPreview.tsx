import { useContext, useState } from 'react';
import { DocumentData, DocumentId, DocumentType, DocumentSubtype, BLOBId } from 'dto';
import { isAttachment, isAudioAttachment, isImageAttachment } from 'utils/schema';
import { getBlobUrl, getScaledImageUrl } from 'utils';
import { Button } from 'components/Button';
import { AudioPlayer } from 'components/AudioPlayer/AudioPlayer';
import { SuspenseImage } from 'components/SuspenseImage';
import { RefClickHandlerContext } from 'components/Ref';
import { Dialog } from 'components/Dialog';

type AttachmentPreviewBlockProps = {
  documentId: DocumentId;
  subtype: DocumentSubtype;
  data: DocumentData;
  description?: string;
};
export function AttachmentPreviewBlock({
  documentId,
  subtype,
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

      <AttachmentPreview subtype={subtype} data={data} />
    </span>
  );
}

type AttachmentPreviewProps = {
  subtype: DocumentSubtype;
  data: DocumentData;
};
export function AttachmentPreview({ subtype, data }: AttachmentPreviewProps) {
  const [showImageModal, setShowImageModal] = useState(false);

  const blobId = data['blob'] as BLOBId;
  const size = data['size'] as number;

  const blobUrl = getBlobUrl(blobId);

  if (isImageAttachment(subtype)) {
    if (showImageModal) {
      return (
        <Dialog
          className="w-fit max-w-full"
          title={data['filename'] as string}
          onHide={() => setShowImageModal(false)}
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
        onClick={() => setShowImageModal(true)}
      />
    );
  }

  if (isAudioAttachment(subtype)) {
    return <AudioPlayer url={blobUrl} title="" artist="" />;
  }

  return null;
}

export function canPreview(documentType: DocumentType, subtype: DocumentSubtype): boolean {
  if (!isAttachment(documentType)) {
    return false;
  }

  return isImageAttachment(subtype) || isAudioAttachment(subtype);
}
