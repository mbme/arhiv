import { DocumentBackref, DocumentType, DocumentId } from 'dto';
import { cx } from 'utils';
import { formatDocumentType } from 'utils/schema';
import { Ref } from 'components/Ref';
import { DateTime } from 'components/DateTime';

type DocumentViewerHeadProps = {
  id?: DocumentId;
  documentType: DocumentType;
  updatedAt: string;
  backrefs: DocumentBackref[];
};

export function DocumentViewerHead({
  id,
  documentType,
  updatedAt,
  backrefs,
}: DocumentViewerHeadProps) {
  return (
    <div className="flex justify-between items-start gap-2 pl-2 mb-6">
      <div
        className={cx('flex flex-col gap-2', {
          'invisible': backrefs.length === 0,
        })}
      >
        {backrefs.length > 0 && <h1 className="section-heading">Linked by:</h1>}
        {backrefs.map((backref) => (
          <Ref
            key={backref.id}
            documentId={backref.id}
            documentType={backref.documentType}
            documentTitle={backref.title}
          />
        ))}
      </div>

      <table className="document-head">
        <tbody>
          {id && (
            <tr>
              <td className="section-heading">id:</td>
              <td>{id}</td>
            </tr>
          )}
          <tr>
            <td className="section-heading">type:</td>
            <td>{formatDocumentType(documentType)}</td>
          </tr>
          <tr>
            <td className="section-heading">modified:</td>
            <td>
              <DateTime datetime={updatedAt} className="whitespace-nowrap" />
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}
