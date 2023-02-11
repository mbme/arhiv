import { useState } from 'preact/hooks';
import { DocumentBackref, DocumentId, DocumentType, DocumentSubtype } from 'dto';
import { cx, copyTextToClipbard } from 'utils';
import { formatDocumentType } from 'utils/schema';
import { useTimeout } from 'utils/hooks';
import { Ref } from 'components/Ref';
import { Button } from 'components/Button';
import { DateTime } from 'components/DateTime';
import { Icon } from 'components/Icon';
import { useCardContext } from '../workspace-reducer';

type DocumentViewerHeadProps = {
  id: DocumentId;
  documentType: DocumentType;
  subtype: DocumentSubtype;
  updatedAt: string;
  backrefs: DocumentBackref[];
  collections: DocumentBackref[];
};

export function DocumentViewerHead({
  id,
  documentType,
  subtype,
  updatedAt,
  backrefs,
  collections,
}: DocumentViewerHeadProps) {
  const { open } = useCardContext();

  const [copied, setCopied] = useState(false);

  useTimeout(
    () => {
      setCopied(false);
    },
    2000,
    copied
  );

  const copyIdToClipboard = () => {
    void copyTextToClipbard(id).then(() => {
      setCopied(true);
    });
  };

  return (
    <div className="flex justify-between pl-2 mb-6">
      <div
        className={cx('flex flex-col gap-2', {
          'invisible': backrefs.length === 0 && collections.length === 0,
        })}
      >
        {collections.length > 0 && <h1 className="section-heading">Collections:</h1>}
        {collections.map((collection) => (
          <Ref
            key={collection.id}
            documentId={collection.id}
            documentType={collection.documentType}
            subtype={collection.subtype}
            documentTitle={collection.title}
            onClick={() => open({ variant: 'document', documentId: collection.id })}
          />
        ))}
        {backrefs.length > 0 && <h1 className="section-heading">Linked by:</h1>}
        {backrefs.map((backref) => (
          <Ref
            key={backref.id}
            documentId={backref.id}
            documentType={backref.documentType}
            subtype={backref.subtype}
            documentTitle={backref.title}
            onClick={() => open({ variant: 'document', documentId: backref.id })}
          />
        ))}
      </div>

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
