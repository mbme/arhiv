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
        <td class="section-heading">id:</td>
        <td>
          <Button
            variant="link"
            className="block font-mono tracking-wide cursor-pointer group"
            title="Copy document id to clipboard"
          >
            {id}
            <svg class="h-5 w-5 inline-block relative left-[-8px] invisible group-hover:visible group-focus:visible">
              <use xlinkHref="#icon-clipboard-copy" />
            </svg>
          </Button>
        </td>
      </tr>
      <tr>
        <td class="section-heading">type:</td>
        <td class="font-semibold">
          {documentType}
          {subtype && <>/ {subtype}</>}
        </td>
      </tr>
      <tr>
        <td class="section-heading">modified:</td>
        <td>
          <DateTime datetime={updatedAt} />
        </td>
      </tr>
    </table>
  );
}
