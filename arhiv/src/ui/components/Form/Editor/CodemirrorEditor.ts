import { Compartment, EditorSelection, EditorState } from '@codemirror/state';
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightSpecialChars,
  keymap,
  placeholder,
  rectangularSelection,
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

type Options = {
  onBlur?: () => void;
  onChange?: () => void;
};

class CodemirrorEditor {
  private readonlyCompartment = new Compartment();
  private editableCompartment = new Compartment();
  private placeholderCompartment = new Compartment();

  private editor: EditorView;

  constructor(
    parent: HTMLElement,
    initialValue: string,
    private options: Options = {},
  ) {
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
          ],
          markdown(),
        ],
      }),
    });
  }

  private onBlur = () => {
    this.options.onBlur?.();
  };

  private onChange = (viewUpdate: ViewUpdate) => {
    if (viewUpdate.docChanged) {
      this.options.onChange?.();
    }
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
    const { state } = this.editor;

    const cursorPos = Math.min(state.selection.main.from, newValue.length);

    this.editor.dispatch({
      changes: { from: 0, to: state.doc.length, insert: newValue },
      selection: {
        anchor: cursorPos,
        head: cursorPos,
      },
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

  replaceSelections(updater: (value: string) => string) {
    const { state } = this.editor;

    const transaction = state.update(
      state.changeByRange((range) => {
        const value = state.sliceDoc(range.from, range.to);

        const newValue = updater(value);

        return {
          changes: {
            from: range.from,
            to: range.to,
            insert: newValue,
          },
          range: EditorSelection.range(range.from + newValue.length, range.from + newValue.length),
          effects: EditorView.scrollIntoView(range.from + newValue.length, { y: 'center' }),
        };
      }),
    );

    this.editor.dispatch(transaction);
  }

  getFirstVisiblePos(viewport: HTMLElement) {
    const { contentHeight } = this.editor;

    const PADDING_TOP_PX = 5;
    const MIN_LINE_HEIGHT_PX = 12;

    const documentTop =
      this.editor.documentTop - viewport.getBoundingClientRect().top - PADDING_TOP_PX;

    if (documentTop >= 0) {
      return undefined;
    }

    if (documentTop + contentHeight <= 0) {
      return undefined;
    }

    const offset = -documentTop;

    let lineBlock = this.editor.lineBlockAtHeight(offset);
    if (lineBlock.bottom - offset < MIN_LINE_HEIGHT_PX) {
      lineBlock = this.editor.lineBlockAtHeight(lineBlock.bottom + MIN_LINE_HEIGHT_PX);
    }

    // point in the center of the line
    return Math.round((lineBlock.from + lineBlock.to) / 2);
  }

  scrollToPos(pos: number) {
    this.editor.dispatch({
      effects: [EditorView.scrollIntoView(pos, { y: 'start' })],
    });
  }

  destroy() {
    this.editor.destroy();
  }
}

export { CodemirrorEditor };
