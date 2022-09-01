import { Compartment, EditorState } from '@codemirror/state';
import {
  DOMEventHandlers,
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

class VEditor {
  private readonlyCompartment = new Compartment();
  private domEventHandlersCompartment = new Compartment();
  private editableCompartment = new Compartment();

  private editor: EditorView;

  constructor(parent: HTMLElement, initialValue: string) {
    this.editor = new EditorView({
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
            this.readonlyCompartment.of(EditorState.readOnly.of(false)),
            this.editableCompartment.of(EditorView.editable.of(true)),
            this.domEventHandlersCompartment.of(EditorView.domEventHandlers({})),
            keymap.of([
              ...defaultKeymap, //
              ...searchKeymap,
              ...historyKeymap,
            ]),
          ],
          markdown(),
        ],
      }),
    });
  }

  focus() {
    this.editor.focus();
  }

  blur() {
    this.editor.contentDOM.blur();
  }

  getValue(): string {
    return this.editor.state.doc.toString();
  }

  setValue(newValue: string) {
    this.editor.dispatch({
      changes: { from: 0, to: this.editor.state.doc.length, insert: newValue },
    });
  }

  setDisabled(disabled: boolean) {
    const editable = !disabled;

    this.editor.dispatch({
      effects: [this.editableCompartment.reconfigure(EditorView.editable.of(editable))],
    });
  }

  setReadonly(readonly: boolean) {
    this.editor.dispatch({
      effects: [this.readonlyCompartment.reconfigure(EditorState.readOnly.of(readonly))],
    });
  }

  setEventHandlers(handlers: DOMEventHandlers<any>) {
    this.editor.dispatch({
      effects: [
        this.domEventHandlersCompartment.reconfigure(EditorView.domEventHandlers(handlers)),
      ],
    });
  }
}
export { VEditor };
