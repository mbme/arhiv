import { DocumentBackref, DocumentType, DocumentSubtype } from 'dto';
import { cx } from 'utils';
import { formatDocumentType } from 'utils/schema';
import { Ref } from 'components/Ref';
import { DateTime } from 'components/DateTime';

type DocumentViewerHeadProps = {
  documentType: DocumentType;
  subtype: DocumentSubtype;
  updatedAt: string;
  backrefs: DocumentBackref[];
};

export function DocumentViewerHead({
  documentType,
  subtype,
  updatedAt,
  backrefs,
}: DocumentViewerHeadProps) {
  return (
    <div className="flex justify-between pl-2 mb-6">
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
            subtype={backref.subtype}
            documentTitle={backref.title}
          />
        ))}
      </div>

      <table id="document-head">
        <tbody>
          <tr>
            <td className="section-heading">type:</td>
            <td>{formatDocumentType(documentType, subtype)}</td>
          </tr>
          <tr>
            <td className="section-heading">modified:</td>
            <td>
              <DateTime datetime={updatedAt} />
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}
