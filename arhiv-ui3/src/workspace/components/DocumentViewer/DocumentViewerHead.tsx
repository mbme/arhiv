import { formatDocumentType } from '../../../scripts/utils';
import { Button } from '../Button';
import { DateTime } from '../DateTime';

type DocumentViewerHeadProps = {
  id: string;
  documentType: string;
  subtype: string;
  updatedAt: string;
};

export function DocumentViewerHead({
  id,
  documentType,
  subtype,
  updatedAt,
}: DocumentViewerHeadProps) {
  return (
    <table id="document-head">
      <tr>
        <td className="section-heading">id:</td>
        <td>
          <Button
            variant="text"
            className="block font-mono tracking-wide cursor-pointer group"
            title="Copy document id to clipboard"
          >
            {id}
            <svg className="h-5 w-5 inline-block relative left-[-8px] invisible group-hover:visible group-focus:visible">
              <use xlinkHref="#icon-clipboard-copy" />
            </svg>
          </Button>
        </td>
      </tr>
      <tr>
        <td className="section-heading">type:</td>
        <td className="font-semibold">{formatDocumentType(documentType, subtype)}</td>
      </tr>
      <tr>
        <td className="section-heading">modified:</td>
        <td>
          <DateTime datetime={updatedAt} />
        </td>
      </tr>
    </table>
  );
}
