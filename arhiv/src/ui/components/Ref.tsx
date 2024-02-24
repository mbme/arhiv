import { createContext, useContext } from 'react';
import { DocumentId, DocumentType, DocumentSubtype } from 'dto';
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

  if (attachmentPreview && canPreview(result.documentType, result.subtype)) {
    return (
      <AttachmentPreviewBlock
        documentId={result.id}
        subtype={result.subtype}
        data={result.data}
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
        isErasedDocument(documentType) && 'line-through text-slate-700/50',
      )}
      onClick={(e) => {
        e.preventDefault();

        refClickHandler(documentId);
      }}
    >
      <DocumentIcon documentType={documentType} className="align-text-bottom mr-1" />

      {description || documentTitle}
    </a>
  );
}
