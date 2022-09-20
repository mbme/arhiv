import { cx, formatDocumentType } from '../../scripts/utils';
import { useQuery } from '../hooks';
import { RPC } from '../rpc';
import { isAttachment } from '../schema';
import { useCardContext } from '../workspace-reducer';
import { Button } from './Button';
import { getAttachmentPreview } from './DocumentViewer/DocumentViewer';
import { QueryError } from './QueryError';

type RefContainerProps = {
  id: string;
  attachmentPreview?: boolean;
  title?: string;
  children?: string;
};
export function RefContainer({ id, attachmentPreview, title, children }: RefContainerProps) {
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

  if (attachmentPreview && isAttachment(result.documentType)) {
    const preview = getAttachmentPreview(result.subtype, result.data);

    if (preview) {
      return preview;
    }
  }

  return (
    <Ref
      id={result.id}
      documentType={result.documentType}
      subtype={result.subtype}
      documentTitle={result.title}
      title={title}
    >
      {children}
    </Ref>
  );
}

type RefProps = {
  id: string;
  documentType: string;
  subtype: string;
  documentTitle?: string;
  title?: string;
  children?: string;
};
export function Ref({ id, documentType, subtype, documentTitle, title, children }: RefProps) {
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
      {children || (
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
