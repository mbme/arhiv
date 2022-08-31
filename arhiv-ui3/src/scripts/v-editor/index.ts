import 'element-internals-polyfill';
import type { ElementInternals } from 'element-internals-polyfill/dist/element-internals';
import { initEditor, EditorView } from './editor';

// TODO: disabled readonly https://stackoverflow.com/a/7730719

export class HTMLVEditorElement extends HTMLElement {
  static get formAssociated() {
    return true;
  }

  static get observedAttributes() {
    return ['required'];
  }

  private editor?: EditorView;
  private internals: ElementInternals;

  constructor() {
    super();

    this.internals = this.attachInternals();

    this.addEventListener('focus', () => this.editor?.focus());
  }

  connectedCallback() {
    this.editor = initEditor(this, this.getInitialValue(), () => {
      this.updateFormValue();
    });
    this.updateFormValue();

    // element must be focusable for form validation to work
    this.tabIndex = 0;

    if (this.hasAttribute('autofocus')) {
      this.editor.focus();
    }
  }

  attributeChangedCallback(name: string) {
    if (name === 'required') {
      this.updateFormValue();
    }
  }

  private updateFormValue = () => {
    const value = this.value;

    if (this.hasAttribute('required') && !value) {
      this.internals.setValidity({ valueMissing: true }, 'Field must not be empty');
    } else {
      this.internals.setValidity({});
    }

    this.internals.setFormValue(value);
  };

  private getInitialValue = () => this.getAttribute('value') ?? this.getDefaultValue();
  private getDefaultValue = () => this.getAttribute('defaultValue') ?? '';

  formDisabledCallback(disabled: boolean) {
    // Do something.  e.g. adding/removing ‘disabled’ content attributes
    // to/from form controls in this shadow tree.
    if (disabled) {
      console.log('is disabled');
    }
  }

  formResetCallback() {
    this.value = this.getDefaultValue();
  }

  formStateRestoreCallback(state: string) {
    this.value = state;
  }

  get form(): HTMLFormElement | null {
    return this.internals.form;
  }

  get name() {
    return this.getAttribute('name');
  }

  get type() {
    return this.localName;
  }

  get value() {
    return this.editor?.state.doc.toString() ?? this.getInitialValue();
  }

  set value(value: string) {
    const editor = this.editor;

    if (!editor) {
      throw new Error("editor isn't initialized yet");
    }

    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert: value },
    });
  }

  get validity() {
    return this.internals.validity;
  }
  get validationMessage() {
    return this.internals.validationMessage;
  }
  get willValidate() {
    return this.internals.willValidate;
  }

  checkValidity() {
    return this.internals.checkValidity();
  }
  reportValidity() {
    return this.internals.reportValidity();
  }
}

declare module 'preact' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace JSX {
    interface EditorElementAttributes extends JSX.HTMLAttributes<HTMLElement> {
      autofocus?: boolean;
      required?: boolean;
      defaultValue?: string;
      value?: string;
    }

    interface IntrinsicElements {
      'v-editor': EditorElementAttributes;
    }
  }
}

customElements.define('v-editor', HTMLVEditorElement);
