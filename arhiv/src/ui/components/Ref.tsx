import { createContext, useContext } from 'react';
import { DocumentData, DocumentId, DocumentType, DocumentSubtype } from 'dto';
import { cx, getDocumentUrl } from 'utils';
import {
  formatDocumentType,
  isAttachment,
  isAudioAttachment,
  isErasedDocument,
  isImageAttachment,
} from 'utils/schema';
import { useSuspenseQuery } from 'utils/suspense';
import { Button } from 'components/Button';
import { AudioPlayer } from 'components/AudioPlayer/AudioPlayer';
import { SuspenseImage } from 'components/SuspenseImage';

export const RefClickHandlerContext = createContext((documentId: DocumentId) => {
  console.log('Ref clicked:', documentId);
});

type RefContainerProps = {
  id: DocumentId;
  description?: string;
  attachmentPreview?: boolean;
};
export function RefContainer({ id, description, attachmentPreview }: RefContainerProps) {
  const { value: result } = useSuspenseQuery({ typeName: 'GetDocument', id });

  if (attachmentPreview) {
    return (
      <RefPreview
        documentId={result.id}
        documentType={result.documentType}
        subtype={result.subtype}
        data={result.data}
        documentTitle={result.title}
        description={description}
      />
    );
  }

  return (
    <Ref
      documentId={result.id}
      documentType={result.documentType}
      subtype={result.subtype}
      documentTitle={result.title}
      description={description}
    />
  );
}

type RefProps = {
  documentId: DocumentId;
  documentType: DocumentType;
  subtype: DocumentSubtype;
  documentTitle: string;
  description?: string;
};
export function Ref({ documentId, documentType, subtype, documentTitle, description }: RefProps) {
  const refClickHandler = useContext(RefClickHandlerContext);

  const typeStr = formatDocumentType(documentType, subtype).toUpperCase();

  return (
    <a
      href={getDocumentUrl(documentId)}
      title={`${typeStr} ${documentTitle}`}
      target="_blank"
      rel="noopen noreferer"
      className={cx(
        'font-semibold text-blue-700 hover:text-blue-600/90 break-words cursor-pointer',
        isErasedDocument(documentType) && 'line-through text-slate-700/50',
      )}
      onClick={(e) => {
        e.preventDefault();

        refClickHandler(documentId);
      }}
    >
      <span
        className="text-[smaller] font-normal font-mono tracking-tight bg-slate-100 px-0.5 mr-1 align-middle"
        hidden={isErasedDocument(documentType)}
      >
        {typeStr}
      </span>
      {description || documentTitle}
    </a>
  );
}

type RefPreviewProps = {
  documentId: DocumentId;
  documentType: DocumentType;
  subtype: DocumentSubtype;
  data: DocumentData;
  documentTitle: string;
  description?: string;
};
export function RefPreview({
  documentId,
  documentType,
  subtype,
  data,
  documentTitle,
  description,
}: RefPreviewProps) {
  const refClickHandler = useContext(RefClickHandlerContext);

  let preview;
  if (isAttachment(documentType)) {
    preview = getAttachmentPreview(subtype, data);
  }

  if (!preview) {
    return (
      <Ref
        documentId={documentId}
        documentType={documentType}
        subtype={subtype}
        documentTitle={documentTitle}
        description={description}
      />
    );
  }

  return (
    <div className="RefPreview w-full group">
      <div className="flex space-between items-center">
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
      </div>
      {preview}
    </div>
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
