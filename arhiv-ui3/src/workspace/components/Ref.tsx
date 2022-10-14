import { cx, formatDocumentType } from '../../scripts/utils';
import { DocumentData } from '../dto';
import { useQuery } from '../hooks';
import { RPC } from '../rpc';
import { isAttachment } from '../schema';
import { useCardContext } from '../workspace-reducer';
import { Button } from './Button';
import { getAttachmentPreview } from './DocumentViewer/DocumentViewer';
import { Icon } from './Icon';
import { QueryError } from './QueryError';

type RefContainerProps = {
  id: string;
  title?: string;
  description?: string;
  attachmentPreview?: boolean;
};
export function RefContainer({ id, title, description, attachmentPreview }: RefContainerProps) {
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
        id={result.id}
        documentType={result.documentType}
        subtype={result.subtype}
        data={result.data}
        documentTitle={result.title}
        title={title}
        description={description}
      />
    );
  }

  return (
    <Ref
      id={result.id}
      documentType={result.documentType}
      subtype={result.subtype}
      documentTitle={result.title}
      title={title}
      description={description}
    />
  );
}

type RefProps = {
  id: string;
  documentType: string;
  subtype: string;
  documentTitle: string;
  title?: string;
  description?: string;
};
export function Ref({ id, documentType, subtype, documentTitle, title, description }: RefProps) {
  const context = useCardContext();

  const openDocument = () => {
    context.open({ variant: 'document', documentId: id });
  };

  return (
    <Button
      variant="text"
      className={cx(
        'bg-yellow-300/30 hover:bg-yellow-300/60 px-1 py-0 rounded w-fit flex items-baseline',
        documentType || 'line-through text-slate-700/50'
      )}
      onClick={openDocument}
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
  id: string;
  documentType: string;
  subtype: string;
  data: DocumentData;
  documentTitle: string;
  title?: string;
  description?: string;
};
export function RefPreview({
  id,
  documentType,
  subtype,
  data,
  documentTitle,
  title,
  description,
}: RefPreviewProps) {
  const context = useCardContext();

  const openDocument = () => {
    context.open({ variant: 'document', documentId: id });
  };

  let preview;
  if (isAttachment(documentType)) {
    preview = getAttachmentPreview(subtype, data);
  }

  if (!preview) {
    return (
      <Ref
        id={id}
        documentType={documentType}
        subtype={subtype}
        documentTitle={documentTitle}
        title={title}
        description={description}
      />
    );
  }

  return (
    <div title={title} className="RefPreview group">
      <div className="flex space-between items-center">
        <span className="text-blue-900 pointer font-serif pl-1">{description}</span>

        <Button
          variant="text"
          onClick={openDocument}
          className="ml-auto text-sm  transition invisible opacity-0 group-hover:visible group-hover:opacity-100"
        >
          open
          <Icon variant="link-arrow" className="h-4 w-4" />
        </Button>
      </div>
      {preview}
    </div>
  );
}
