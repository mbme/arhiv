import { createContext, useContext } from 'react';
import { DocumentId, DocumentType } from 'dto';
import { cx, getDocumentUrl } from 'utils';
import { formatDocumentType, isErasedDocument } from 'utils/schema';
import { useSuspenseQuery } from 'utils/suspense';
import { AttachmentPreviewBlock, canPreview } from 'components/AttachmentPreview';
import { DocumentIcon } from './DocumentIcon';

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

  if (attachmentPreview && canPreview(result.documentType, result.data)) {
    return (
      <AttachmentPreviewBlock documentId={result.id} data={result.data} description={description} />
    );
  }

  return (
    <Ref
      documentId={result.id}
      documentType={result.documentType}
      documentTitle={result.title}
      description={description}
    />
  );
}

type RefProps = {
  documentId: DocumentId;
  documentType: DocumentType;
  documentTitle: string;
  description?: string;
};
export function Ref({ documentId, documentType, documentTitle, description }: RefProps) {
  const refClickHandler = useContext(RefClickHandlerContext);

  const typeStr = formatDocumentType(documentType).toUpperCase();

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
      <DocumentIcon documentType={documentType} className="align-text-bottom" />
      &nbsp;
      {description || documentTitle}
    </a>
  );
}
