import { EditorState } from '@codemirror/state';
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightSpecialChars,
  keymap,
} from '@codemirror/view';
import { markdown } from '@codemirror/lang-markdown';
import { lineNumbers } from '@codemirror/gutter';
import { defaultKeymap } from '@codemirror/commands';
import { history, historyKeymap } from '@codemirror/history';
import { indentOnInput } from '@codemirror/language';
import { bracketMatching } from '@codemirror/matchbrackets';
import { closeBrackets, closeBracketsKeymap } from '@codemirror/closebrackets';
import { rectangularSelection } from '@codemirror/rectangular-selection';
import { highlightSelectionMatches, searchKeymap } from '@codemirror/search';
import { classHighlightStyle, defaultHighlightStyle } from '@codemirror/highlight';

export function initEditor(textareaEl: HTMLTextAreaElement): EditorView {
  const parentEl = textareaEl.parentElement;
  if (!parentEl) {
    throw new Error('textarea must have a parent element');
  }

  const handlers = EditorView.domEventHandlers({
    'blur': (_event, view) => {
      textareaEl.value = view.state.doc.toString();
    },
  });

  const editor = new EditorView({
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
          defaultHighlightStyle.extension,
          classHighlightStyle.extension,
          EditorView.lineWrapping,
          bracketMatching(),
          closeBrackets(),
          rectangularSelection(),
          highlightSelectionMatches(),
          keymap.of([
            ...closeBracketsKeymap, //
            ...defaultKeymap,
            ...searchKeymap,
            ...historyKeymap,
          ]),
        ],
        handlers, //
        markdown(),
      ],
    }),
  });

  parentEl.insertBefore(editor.dom, textareaEl);

  textareaEl.setAttribute('hidden', 'true');

  return editor;
}
