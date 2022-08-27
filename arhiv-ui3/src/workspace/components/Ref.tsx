import { formatDocumentType } from '../../scripts/utils';
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
};
export function RefContainer({ id, attachmentPreview }: RefContainerProps) {
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
      title={result.title}
    />
  );
}

type RefProps = {
  id: string;
  documentType: string;
  subtype: string;
  title: string;
};
export function Ref({ id, documentType, subtype, title }: RefProps) {
  const context = useCardContext();

  const openDocument = () => {
    context.open({ variant: 'document', documentId: id });
  };

  return (
    <Button
      variant="text"
      className="bg-yellow-300 bg-opacity-30 px-2 py-1 rounded-sm"
      onClick={openDocument}
    >
      <span className="font-mono uppercase text-gray-400 mr-4">
        {formatDocumentType(documentType, subtype)}
      </span>
      {title}
    </Button>
  );
}
