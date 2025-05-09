import { DocumentBackref, DocumentType, DocumentId } from 'dto';
import { cx } from 'utils';
import { formatDocumentType } from 'utils/schema';
import { Ref } from 'components/Ref';
import { DateTime } from 'components/DateTime';
import { Spoiler } from 'components/Spoiler';

type DocumentViewerHeadProps = {
  id?: DocumentId;
  documentType: DocumentType;
  updatedAt: string;
  backrefs: DocumentBackref[];
  snapshotsCount: number;
};

export function DocumentViewerHead({
  id,
  documentType,
  updatedAt,
  backrefs,
  snapshotsCount,
}: DocumentViewerHeadProps) {
  const valueNameClass = 'pb-1 tracking-wide';
  const rowNameClass = 'section-heading pb-0 tracking-wide text-right pb-0 pr-1 align-middle';

  return (
    <div className="flex flex-col gap-2 mb-6">
      <table className="relative text-sm grow-0 shrink-0 self-end">
        <tbody>
          {id && (
            <tr>
              <td className={rowNameClass}>id:</td>
              <td className={valueNameClass}>{id}</td>
            </tr>
          )}
          <tr>
            <td className={rowNameClass}>type:</td>
            <td className={valueNameClass}>{formatDocumentType(documentType)}</td>
          </tr>
          <tr>
            <td className={rowNameClass}>modified:</td>
            <td className={valueNameClass}>
              <DateTime datetime={updatedAt} className="whitespace-nowrap" />
            </td>
          </tr>
          <tr>
            <td className={rowNameClass}>snapshots:</td>
            <td className={valueNameClass}>{snapshotsCount}</td>
          </tr>
        </tbody>
      </table>

      <Spoiler
        heading={<h1 className="section-heading">Linked by {backrefs.length} documents</h1>}
        className={cx({
          'invisible': backrefs.length === 0,
        })}
      >
        <div className="flex flex-col gap-2 -mr-4">
          {backrefs.map((backref) => (
            <Ref
              key={backref.id}
              documentId={backref.id}
              documentType={backref.documentType}
              documentTitle={backref.title}
            />
          ))}
        </div>
      </Spoiler>
    </div>
  );
}
