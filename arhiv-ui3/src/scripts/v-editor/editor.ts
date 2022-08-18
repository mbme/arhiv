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
import { Callback } from '../utils';

export type Editor = EditorView;

export function initEditor(parent: HTMLElement, initialValue: string, onBlur?: Callback): Editor {
  const handlers = EditorView.domEventHandlers({
    'blur': (_event, _view) => {
      onBlur?.();
    },
  });

  const editor = new EditorView({
    parent,
    state: EditorState.create({
      doc: initialValue,
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

  return editor;
}

export { EditorView };
