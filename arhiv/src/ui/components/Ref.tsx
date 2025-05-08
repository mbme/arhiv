import { createContext, useContext } from 'react';
import { DocumentId, DocumentType } from 'dto';
import { cx } from 'utils';
import { getDocumentUrl } from 'utils/network';
import { formatDocumentType, isErasedDocument } from 'utils/schema';
import { useCachedRef } from 'controller';
import { AssetPreviewBlock, canPreview } from 'components/AssetPreview';
import { DocumentIcon } from './DocumentIcon';

export const RefClickHandlerContext = createContext((documentId: DocumentId) => {
  console.log('Ref clicked:', documentId);
});

type RefContainerProps = {
  id: DocumentId;
  description?: string;
  assetPreview?: boolean;
};
export function RefContainer({ id, description, assetPreview }: RefContainerProps) {
  const result = useCachedRef(id);

  if (assetPreview && canPreview(result.documentType, result.data)) {
    return <AssetPreviewBlock documentId={id} data={result.data} description={description} />;
  }

  return (
    <Ref
      documentId={id}
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
      rel="noreferrer"
      className={cx(
        'font-semibold var-active-color var-active-color-hover break-words cursor-pointer',
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
