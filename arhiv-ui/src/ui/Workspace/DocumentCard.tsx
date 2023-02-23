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

  if (error) {
    return (
      <CardContainer>
        <QueryError error={error} />
      </CardContainer>
    );
  }

  if (inProgress || !result) {
    return (
      <CardContainer>
        <Icon variant="spinner" className="mb-8" />
      </CardContainer>
    );
  }

  if (edit) {
    return (
      <DocumentEditor
        document={result}
        onSave={() => {
          setEdit(false);
          triggerRefresh();
        }}
        onCancel={() => setEdit(false)}
      />
    );
  }

  return (
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
