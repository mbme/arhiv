import { DocumentType } from 'dto';
import { DocumentIcon } from 'components/DocumentIcon';

type Props = {
  documentType: DocumentType;
  title: string;
};
export function DocumentTitle({ documentType, title }: Props) {
  return (
    <span title={`${documentType.toUpperCase()} ${title}`}>
      {documentType && <DocumentIcon className="size-4 mr-1 mb-1" documentType={documentType} />}
      {title}
    </span>
  );
}
