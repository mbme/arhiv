import { Callback, cx, formatDocumentType } from '../utils';
import { DocumentData } from '../../dto';
import { useQuery } from '../utils/hooks';
import { RPC } from '../utils/rpc';
import { isAttachment, isAudioAttachment, isImageAttachment } from '../utils/schema';
import { Button } from './Button';
import { QueryError } from './QueryError';
import { AudioPlayer } from './AudioPlayer/AudioPlayer';

type RefContainerProps = {
  id: string;
  title?: string;
  description?: string;
  attachmentPreview?: boolean;
  onClick: Callback;
};
export function RefContainer({
  id,
  title,
  description,
  attachmentPreview,
  onClick,
}: RefContainerProps) {
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
        documentType={result.documentType}
        subtype={result.subtype}
        data={result.data}
        documentTitle={result.title}
        title={title}
        description={description}
        onClick={onClick}
      />
    );
  }

  return (
    <Ref
      documentType={result.documentType}
      subtype={result.subtype}
      documentTitle={result.title}
      title={title}
      description={description}
      onClick={onClick}
    />
  );
}

type RefProps = {
  documentType: string;
  subtype: string;
  documentTitle: string;
  title?: string;
  description?: string;
  onClick: Callback;
};
export function Ref({
  documentType,
  subtype,
  documentTitle,
  title,
  description,
  onClick,
}: RefProps) {
  return (
    <Button
      variant="text"
      className={cx(
        'bg-yellow-300/30 hover:bg-yellow-300/60 px-1 py-0 rounded w-fit flex items-baseline',
        documentType || 'line-through text-slate-700/50'
      )}
      onClick={onClick}
      title={title}
    >
      {description || (
        <>
          <span className="font-mono uppercase text-gray-400 mr-1">
            {formatDocumentType(documentType, subtype)}
          </span>
          {documentTitle}
        </>
      )}
    </Button>
  );
}

type RefPreviewProps = {
  documentType: string;
  subtype: string;
  data: DocumentData;
  documentTitle: string;
  title?: string;
  description?: string;
  onClick: Callback;
};
export function RefPreview({
  documentType,
  subtype,
  data,
  documentTitle,
  title,
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
        documentType={documentType}
        subtype={subtype}
        documentTitle={documentTitle}
        title={title}
        description={description}
        onClick={onClick}
      />
    );
  }

  return (
    <div title={title} className="RefPreview group">
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
