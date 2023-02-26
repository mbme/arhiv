import { Suspense } from 'preact/compat';
import { useState } from 'preact/hooks';
import { DocumentDTO } from 'dto';
import { Callback, copyTextToClipbard, getDocumentUrl } from 'utils';
import { getFieldDescriptions, isAttachment, isErasedDocument } from 'utils/schema';
import { Icon } from 'components/Icon';
import { IconButton } from 'components/Button';
import { getAttachmentPreview } from 'components/Ref';
import { DropdownMenu } from 'components/DropdownMenu';
import { CardContainer } from '../CardContainer';
import { DocumentViewerHead } from './DocumentViewerHead';
import { EraseDocumentConfirmationDialog } from './EraseDocumentConfirmationDialog';
import { DocumentViewerField, FieldValue } from './DocumentViewerField';

type DocumentViewerProps = {
  document: DocumentDTO;
  onEdit: Callback;
  onClone: Callback;
  onErase: Callback;
};

export function DocumentViewer({ document, onEdit, onClone, onErase }: DocumentViewerProps) {
  const [showEraseDocumentConfirmationDialog, setShowEraseDocumentConfirmationDialog] =
    useState(false);

  const fields = getFieldDescriptions(document.documentType, document.subtype);

  return (
    <CardContainer>
      <CardContainer.Topbar
        left={
          <>
            <DropdownMenu
              icon="dots-horizontal"
              options={[
                {
                  text: 'Copy link',
                  icon: 'clipboard',
                  onClick: () => {
                    void copyTextToClipbard(getDocumentUrl(document.id));
                  },
                },
                {
                  text: `Clone ${document.documentType}`,
                  icon: 'duplicate-document',
                  onClick: onClone,
                },
                {
                  text: `Erase ${document.documentType}`,
                  icon: 'erase-document',
                  alarming: true,
                  onClick: () => setShowEraseDocumentConfirmationDialog(true),
                },
              ]}
            />

            <IconButton
              icon="pencil-square"
              title={`Edit ${document.documentType}`}
              onClick={onEdit}
              size="lg"
            />
          </>
        }
        right={<CardContainer.CloseButton />}
      />

      <DocumentViewerHead
        id={document.id}
        documentType={document.documentType}
        subtype={document.subtype}
        updatedAt={document.updatedAt}
        backrefs={document.backrefs}
        collections={document.collections}
      />

      {isAttachment(document.documentType) && (
        <div className="mb-8 empty:hidden">
          {getAttachmentPreview(document.subtype, document.data)}
        </div>
      )}

      {isErasedDocument(document.documentType) && (
        <img
          src="/public/nothing-to-see-here.jpg"
          alt="funny picture for the erased document"
          className="my-16 mx-auto"
        />
      )}

      <Suspense fallback={<Icon variant="spinner" className="mb-8" />}>
        <div className="divide-y divide-dashed">
          {fields.map((field) => {
            const value = document.data[field.name];

            if (value === null || value === undefined) {
              return null;
            }

            return (
              <DocumentViewerField key={field.name} name={field.name}>
                <FieldValue field={field} value={value} />
              </DocumentViewerField>
            );
          })}
        </div>
      </Suspense>

      {showEraseDocumentConfirmationDialog && (
        <EraseDocumentConfirmationDialog
          documentId={document.id}
          documentType={document.documentType}
          title={document.title}
          onErase={onErase}
          onCancel={() => setShowEraseDocumentConfirmationDialog(false)}
        />
      )}
    </CardContainer>
  );
}
