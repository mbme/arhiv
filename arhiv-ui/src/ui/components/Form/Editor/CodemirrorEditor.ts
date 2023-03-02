import { Compartment, EditorState } from '@codemirror/state';
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightSpecialChars,
  keymap,
  placeholder,
  rectangularSelection,
  showPanel,
  ViewUpdate,
} from '@codemirror/view';
import { markdown } from '@codemirror/lang-markdown';
import {
  cursorLineEnd,
  cursorLineStart,
  defaultKeymap,
  deleteToLineEnd,
  history,
  historyKeymap,
} from '@codemirror/commands';
import {
  indentOnInput,
  bracketMatching,
  defaultHighlightStyle,
  syntaxHighlighting,
} from '@codemirror/language';
import { createToolbar } from './EditorToolbar';

type Options = {
  onBlur?: () => void;
  onChange?: () => void;
};

class CodemirrorEditor {
  private readonlyCompartment = new Compartment();
  private editableCompartment = new Compartment();
  private placeholderCompartment = new Compartment();

  private editor: EditorView;

  constructor(parent: HTMLElement, initialValue: string, private options: Options = {}) {
    this.editor = new EditorView({
      parent,
      state: EditorState.create({
        doc: initialValue,
        extensions: [
          [
            highlightActiveLine(),
            highlightSpecialChars(),
            history(),
            drawSelection(),
            indentOnInput(),
            syntaxHighlighting(defaultHighlightStyle),
            EditorView.lineWrapping,
            bracketMatching(),
            rectangularSelection(),
            this.readonlyCompartment.of(EditorState.readOnly.of(false)),
            this.editableCompartment.of(EditorView.editable.of(true)),
            this.placeholderCompartment.of(placeholder('')),
            EditorView.domEventHandlers({ 'blur': this.onBlur }),
            EditorView.updateListener.of((viewUpdate) => {
              this.onChange(viewUpdate);
            }),
            keymap.of([
              ...defaultKeymap, //
              ...historyKeymap,
              { key: 'Ctrl-a', run: cursorLineStart },
              { key: 'Ctrl-e', run: cursorLineEnd },
              { key: 'Ctrl-k', run: deleteToLineEnd },
            ]),
            showPanel.of(createToolbar),
          ],
          markdown(),
        ],
      }),
    });
  }

  private onBlur = () => {
    this.options.onBlur?.();
  };

  private onChange = (_viewUpdate: ViewUpdate) => {
    this.options.onChange?.();
  };

  focus() {
    this.editor.focus();
  }

  isFocused() {
    return this.editor.hasFocus;
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

  setPlaceholder(value: string) {
    this.editor.dispatch({
      effects: [this.placeholderCompartment.reconfigure(placeholder(value))],
    });
  }

  destroy() {
    this.editor.destroy();
  }
}

export { CodemirrorEditor };
