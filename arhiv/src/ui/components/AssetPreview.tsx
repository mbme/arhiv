import { useContext, useState } from 'react';
import { DocumentData, DocumentId, DocumentType } from 'dto';
import { isAsset } from 'utils/schema';
import { isAudio, isImage } from 'utils';
import { getAssetUrl, getScaledImageUrl } from 'utils/network';
import { Button } from 'components/Button';
import { AudioPlayer } from 'components/AudioPlayer/AudioPlayer';
import { SuspenseImage } from 'components/SuspenseImage';
import { RefClickHandlerContext } from 'components/Ref';
import { Dialog } from 'components/Dialog';

type AssetPreviewBlockProps = {
  documentId: DocumentId;
  data: DocumentData;
  description?: string;
};
export function AssetPreviewBlock({ documentId, data, description }: AssetPreviewBlockProps) {
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
          className="ml-auto text-sm transition hoverable:invisible hoverable:opacity-0 group-hover:visible group-hover:opacity-100"
          trailingIcon="link-arrow"
          size="sm"
        >
          open
        </Button>
      </span>

      <AssetPreview assetId={documentId} data={data} />
    </span>
  );
}

type AssetPreviewProps = {
  assetId: DocumentId;
  data: DocumentData;
};
export function AssetPreview({ assetId, data }: AssetPreviewProps) {
  const [showImageModal, setShowImageModal] = useState(false);

  const size = data['size'] as number;
  const mediaType = data['media_type'] as string;

  const assetUrl = getAssetUrl(assetId);

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
          <img className="max-w-full mx-auto" src={assetUrl} />
        </Dialog>
      );
    }

    return (
      <SuspenseImage
        src={getImageUrl(assetId, size)}
        alt=""
        className="max-h-96 mx-auto"
        onClick={() => {
          setShowImageModal(true);
        }}
      />
    );
  }

  if (isAudio(mediaType)) {
    return <AudioPlayer url={assetUrl} title="" artist="" />;
  }

  return null;
}

export function getImageUrl(assetId: DocumentId, size: number): string {
  const assetUrl = getAssetUrl(assetId);
  const compressedImage = getScaledImageUrl(assetId, 600, 500);

  return size < 1_000_000 ? assetUrl : compressedImage;
}

export function getPreviewType(
  documentType: DocumentType,
  data: DocumentData,
): 'image' | 'audio' | null {
  if (!isAsset(documentType)) {
    return null;
  }

  const mediaType = data['media_type'] as string;

  if (isImage(mediaType)) {
    return 'image';
  }

  if (isAudio(mediaType)) {
    return 'audio';
  }

  return null;
}

export function canPreview(documentType: DocumentType, data: DocumentData): boolean {
  return getPreviewType(documentType, data) !== null;
}
