import { useState } from 'preact/hooks';
import { DocumentId } from 'dto';
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

  if (edit) {
    return (
      <CardContainer>
        <DocumentEditor
          key={documentId}
          documentId={documentId}
          onSave={() => setEdit(false)}
          onCancel={() => setEdit(false)}
        />
      </CardContainer>
    );
  }

  return (
    <CardContainer>
      <DocumentViewer
        key={documentId}
        documentId={documentId}
        onEdit={() => setEdit(true)}
        onClone={(documentType, subtype, data) => {
          context.open({ variant: 'new-document', documentType, subtype, data });
        }}
      />
    </CardContainer>
  );
}
