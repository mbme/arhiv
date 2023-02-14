import { useState } from 'preact/hooks';
import { DocumentId } from 'dto';
import { useQuery } from 'utils/hooks';
import { RPC } from 'utils/rpc';
import { QueryError } from 'components/QueryError';
import { Icon } from 'components/Icon';
import { DocumentEditor } from './DocumentEditor/DocumentEditor';
import { DocumentViewer } from './DocumentViewer/DocumentViewer';
import { useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';

type Props = {
  documentId: DocumentId;
};

export function DocumentCard({ documentId }: Props) {
  const context = useCardContext();

  const [edit, setEdit] = useState(false);

  const { result, error, inProgress, triggerRefresh } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    {
      refreshIfChange: [documentId],
    }
  );

  let content;
  if (error) {
    content = <QueryError error={error} />;
  } else if (inProgress || !result) {
    content = <Icon variant="spinner" className="mb-8" />;
  } else if (edit) {
    content = (
      <DocumentEditor
        document={result}
        onSave={() => setEdit(false)}
        onCancel={() => setEdit(false)}
      />
    );
  } else {
    content = (
      <DocumentViewer
        document={result}
        onEdit={() => setEdit(true)}
        onClone={() => {
          context.open({
            variant: 'new-document',
            documentType: result.documentType,
            subtype: result.subtype,
            data: result.data,
          });
        }}
        onErase={triggerRefresh}
      />
    );
  }

  return <CardContainer>{content}</CardContainer>;
}
