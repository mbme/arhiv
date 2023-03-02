import { render } from 'preact';
import { useState } from 'preact/hooks';
import { EditorView } from '@codemirror/view';
import { EditorSelection } from '@codemirror/state';
import { DocumentId, DocumentSubtype, DocumentType } from 'dto';
import { createLink, createRefUrl } from 'utils/markup';
import { canPreview } from 'components/Ref';
import { Button } from 'components/Button';
import { DocumentPicker } from 'components/DocumentPicker';

type Props = {
  onDocumentSelected: (
    id: DocumentId,
    documentType: DocumentType,
    subtype: DocumentSubtype
  ) => void;
};

function EditorToolbar({ onDocumentSelected }: Props) {
  const [showPicker, setShowPicker] = useState(false);

  return (
    <>
      <Button leadingIcon="link" variant="text" size="sm" onClick={() => setShowPicker(true)}>
        Add ref
      </Button>

      {showPicker && (
        <DocumentPicker
          hideOnSelect
          onSelected={(documentId, documentType, subtype) => {
            setShowPicker(false);
            onDocumentSelected(documentId, documentType, subtype);
          }}
          onCancel={() => setShowPicker(false)}
        />
      )}
    </>
  );
}

export function createToolbar(view: EditorView) {
  const dom = document.createElement('div');
  dom.classList.add('v-editor-toolbar');

  render(
    <EditorToolbar
      onDocumentSelected={(id, documentType, subtype) => {
        const { state } = view;

        const transaction = state.update(
          state.changeByRange((range) => {
            const value = state.sliceDoc(range.from, range.to);

            const newValue = createLink(createRefUrl(id), value, canPreview(documentType, subtype));

            return {
              changes: {
                from: range.from,
                to: range.to,
                insert: newValue,
              },
              range: EditorSelection.range(
                range.from + newValue.length,
                range.from + newValue.length
              ),
              effects: EditorView.scrollIntoView(range.from + newValue.length, { y: 'center' }),
            };
          })
        );

        view.dispatch(transaction);

        view.focus();
      }}
    />,
    dom
  );

  return { dom };
}
