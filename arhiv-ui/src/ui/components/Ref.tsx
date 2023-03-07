import { createContext } from 'preact';
import { DocumentData, DocumentId, DocumentType, DocumentSubtype } from 'dto';
import { cx, getDocumentUrl } from 'utils';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import {
  formatDocumentType,
  isAttachment,
  isAudioAttachment,
  isErasedDocument,
  isImageAttachment,
} from 'utils/schema';
import { Button } from 'components/Button';
import { QueryError } from 'components/QueryError';
import { AudioPlayer } from 'components/AudioPlayer/AudioPlayer';
import { useContext } from 'preact/hooks';

export const RefClickHandlerContext = createContext((documentId: DocumentId) => {
  console.log('Ref clicked:', documentId);
});

type RefContainerProps = {
  id: DocumentId;
  description?: string;
  attachmentPreview?: boolean;
};
export function RefContainer({ id, description, attachmentPreview }: RefContainerProps) {
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

export const useDocuments = (ids: DocumentId[]) => {
  const { result, error, inProgress } = useQuery(
    async (abortSignal) => {
      if (ids.length > 0) {
        return RPC.GetDocuments({ ids }, abortSignal);
      }
    },
    {
      refreshIfChange: ids,
    }
  );

  return { inProgress, error, documents: result?.documents };
};

type RefListContainerProps = {
  ids: DocumentId[];
};
export function RefListContainer({ ids }: RefListContainerProps) {
  const { documents, error, inProgress } = useDocuments(ids);

  if (error) {
    return <QueryError error={error} />;
  }

  if (inProgress || !documents) {
    return null;
  }

  return (
    <>
      {documents.map((item) => (
        <Ref
          key={item.id}
          documentId={item.id}
          documentType={item.documentType}
          subtype={item.subtype}
          documentTitle={item.title}
        />
      ))}
    </>
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
        documentType || 'line-through text-slate-700/50'
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
    <div
      className="RefPreview w-full group"
      title={`${formatDocumentType(documentType, subtype).toUpperCase()} ${documentTitle}`}
    >
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
