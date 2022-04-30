import { EditorState } from '@codemirror/state';
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightSpecialChars,
  keymap,
  lineNumbers,
  rectangularSelection,
} from '@codemirror/view';
import { markdown } from '@codemirror/lang-markdown';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import {
  indentOnInput,
  bracketMatching,
  defaultHighlightStyle,
  syntaxHighlighting,
} from '@codemirror/language';
import { highlightSelectionMatches, searchKeymap } from '@codemirror/search';

export function initEditor(textareaEl: HTMLTextAreaElement, parent: HTMLElement): EditorView {
  const handlers = EditorView.domEventHandlers({
    'blur': (_event, view) => {
      textareaEl.value = view.state.doc.toString();
    },
  });

  const editor = new EditorView({
    parent,
    state: EditorState.create({
      doc: textareaEl.value,
      extensions: [
        [
          lineNumbers(),
          highlightActiveLine(),
          highlightSpecialChars(),
          history(),
          drawSelection(),
          EditorState.allowMultipleSelections.of(true),
          indentOnInput(),
          syntaxHighlighting(defaultHighlightStyle),
          EditorView.lineWrapping,
          bracketMatching(),
          rectangularSelection(),
          highlightSelectionMatches(),
          keymap.of([
            ...defaultKeymap, //
            ...searchKeymap,
            ...historyKeymap,
          ]),
        ],
        handlers,
        markdown(),
      ],
    }),
  });

  textareaEl.setAttribute('hidden', 'true');

  return editor;
}

export { EditorView };
