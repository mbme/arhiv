import { copyTextToClipbard } from 'utils';
import { useBlobUrl } from 'utils/hooks';
import { DownloadLink } from 'components/Link';
import { Button } from 'components/Button';
import { showToast } from 'components/Toaster';
import { ExportedKey } from './ExportKeyForm';

interface Props {
  exportedKey: ExportedKey;
}
export function ExportKeyView({ exportedKey }: Props) {
  const { key, qrcodeSvgBase64, htmlPage } = exportedKey;

  const imgUrl = useBlobUrl(atob(qrcodeSvgBase64), 'image/svg+xml');
  const keyUrl = useBlobUrl(key, 'text/plain');
  const htmlPageUrl = useBlobUrl(htmlPage, 'text/plain');

  return (
    <>
      <h1 className="heading-1 text-center mb-4">Encrypted Arhiv key</h1>

      <img className="mb-2" src={imgUrl} />

      <div className="mb-4">
        <DownloadLink url={imgUrl ?? ''} fileName="arhiv-key.svg" title="Download QR Code image" />
      </div>

      <hr className="mb-8" />

      <pre className="text-sm mb-4 overflow-x-auto">
        <code>{key}</code>
      </pre>

      <div className="flex justify-between mb-8">
        <DownloadLink url={keyUrl ?? ''} fileName="arhiv-key.age" title="Download Arhiv key" />

        <Button
          leadingIcon="clipboard"
          variant="text"
          onClick={() => {
            void copyTextToClipbard(key).then(() => {
              showToast({
                level: 'info',
                message: 'Copied encrypted Arhiv key to clipboard!',
              });
            });
          }}
        >
          Copy Arhiv key to clipboard
        </Button>
      </div>

      <div>
        <DownloadLink
          url={htmlPageUrl ?? ''}
          fileName="arhiv-key.html"
          title="Download printable HTML page"
        />
      </div>
    </>
  );
}
