import { useState } from 'preact/hooks';
import { cx, copyTextToClipbard } from '../../../scripts/utils';
import { useTimeout } from '../../hooks';
import { Button } from '../Button';
import { DateTime } from '../DateTime';
import { Icon } from '../Icon';

type DocumentViewerHeadProps = {
  id: string;
  updatedAt: string;
};

export function DocumentViewerHead({ id, updatedAt }: DocumentViewerHeadProps) {
  const [copied, setCopied] = useState(false);

  useTimeout(
    () => {
      setCopied(false);
    },
    2000,
    copied
  );

  const copyIdToClipboard = () => {
    void copyTextToClipbard(id).then(
      () => {
        setCopied(true);
        console.log('Copied text "%s" to clipboard"', id);
      },
      (e) => {
        console.error(`Failed to copy text "${id}" to clipboard`, e);
      }
    );
  };

  return (
    <table id="document-head">
      <tbody>
        <tr>
          <td className="section-heading">id:</td>
          <td>
            <Button
              variant="text"
              className="block font-mono tracking-wide cursor-pointer group"
              title="Copy document id to clipboard"
              onClick={copyIdToClipboard}
            >
              {id}
              <Icon
                variant={copied ? 'clipboard-check' : 'clipboard'}
                className={cx('ml-1', {
                  'invisible group-hover:visible': !copied,
                })}
              />
            </Button>
          </td>
        </tr>
        <tr>
          <td className="section-heading">modified:</td>
          <td>
            <DateTime datetime={updatedAt} />
          </td>
        </tr>
      </tbody>
    </table>
  );
}
