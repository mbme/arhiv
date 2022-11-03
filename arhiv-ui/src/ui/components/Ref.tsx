import { DocumentData } from 'dto';
import { Callback, cx, formatDocumentType, getDocumentUrl } from 'utils';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { isAttachment, isAudioAttachment, isImageAttachment } from 'utils/schema';
import { Button } from './Button';
import { QueryError } from './QueryError';
import { AudioPlayer } from './AudioPlayer/AudioPlayer';

type RefContainerProps = {
  id: string;
  description?: string;
  attachmentPreview?: boolean;
  onClick: Callback;
};
export function RefContainer({ id, description, attachmentPreview, onClick }: RefContainerProps) {
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.GetDocument({ id }, abortSignal),
    {
      refreshIfChange: [id],
    }
  );

  if (error) {
    return <QueryError error={error} />;
  }

  if (inProgress || !result) {
    return null;
  }

  if (attachmentPreview) {
    return (
      <RefPreview
        documentId={result.id}
        documentType={result.documentType}
        subtype={result.subtype}
        data={result.data}
        documentTitle={result.title}
        description={description}
        onClick={onClick}
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
      onClick={onClick}
    />
  );
}

type RefProps = {
  documentId: string;
  documentType: string;
  subtype: string;
  documentTitle: string;
  description?: string;
  onClick: Callback;
};
export function Ref({
  documentId,
  documentType,
  subtype,
  documentTitle,
  description,
  onClick,
}: RefProps) {
  return (
    <a
      href={getDocumentUrl(documentId)}
      title={`${formatDocumentType(documentType, subtype).toUpperCase()} ${documentTitle}`}
      target="_blank"
      rel="noopen noreferer"
      className={cx(
        'font-semibold text-blue-700 hover:text-blue-600/90 break-words cursor-pointer',
        documentType || 'line-through text-slate-700/50'
      )}
      onClick={(e) => {
        e.preventDefault();

        onClick();
      }}
    >
      {description || documentTitle}
    </a>
  );
}

type RefPreviewProps = {
  documentId: string;
  documentType: string;
  subtype: string;
  data: DocumentData;
  documentTitle: string;
  description?: string;
  onClick: Callback;
};
export function RefPreview({
  documentId,
  documentType,
  subtype,
  data,
  documentTitle,
  description,
  onClick,
}: RefPreviewProps) {
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
        onClick={onClick}
      />
    );
  }

  return (
    <div
      className="RefPreview w-full group"
      title={`${formatDocumentType(documentType, subtype).toUpperCase()} ${documentTitle}`}
    >
      <div className="flex space-between items-center">
        <span className="text-blue-900 pointer font-serif pl-1">{description}</span>

        <Button
          variant="text"
          onClick={onClick}
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

export function getAttachmentPreview(subtype: string, data: DocumentData) {
  const filename = data['filename'] as string;
  const blobId = data['blob'] as string;

  const blobUrl = `/blobs/${blobId}`;

  if (isImageAttachment(subtype)) {
    return <img src={blobUrl} alt={filename} className="max-h-96 mx-auto" />;
  }

  if (isAudioAttachment(subtype)) {
    return <AudioPlayer url={blobUrl} title="" artist="" />;
  }

  return null;
}
