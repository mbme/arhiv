import { VEditor } from './editor';
import { FormControlElement } from './FormControlElement';

export class HTMLVEditorElement extends FormControlElement {
  static get observedAttributes() {
    return ['required', 'readonly', 'disabled', 'placeholder'];
  }

  private editor?: VEditor;

  constructor() {
    super();

    this.addEventListener('focus', () => {
      this.editor?.focus();
    });
  }

  override connectedCallback() {
    this.editor = new VEditor(this, this.getInitialValue());

    this.editor.setEventHandlers({
      'blur': () => {
        this.updateFormValue();
      },
    });

    this.updateFormValue();
    this.updateState();

    if (this.hasAttribute('autofocus')) {
      this.editor.focus();
    }
  }

  override attributeChangedCallback() {
    this.updateState();
    this.updateFormValue();
  }

  private updateState = () => {
    if (!this.editor) {
      return;
    }

    this.editor.setDisabled(this.disabled);
    this.editor.setReadonly(this.readonly);
    this.editor.setPlaceholder(this.placeholder ?? '');

    if (this.disabled) {
      this.tabIndex = -1;
    } else {
      // element must be focusable for form validation to work
      this.tabIndex = 0;
    }
  };

  private updateFormValue = () => {
    if (this.readonly || this.disabled) {
      return;
    }

    const value = this.value;

    if (this.required && !value) {
      this.internals.setValidity({ valueMissing: true }, 'Field must not be empty');
    } else {
      this.internals.setValidity({});
    }

    this.internals.setFormValue(value);
  };

  private getInitialValue = () => this.getAttribute('value') ?? this.getDefaultValue();
  private getDefaultValue = () => this.getAttribute('defaultValue') ?? '';

  get disabled() {
    return this.hasAttribute('disabled');
  }

  get readonly() {
    return this.hasAttribute('readonly');
  }

  get required() {
    return this.hasAttribute('required');
  }

  get placeholder() {
    return this.getAttribute('placeholder');
  }

  override formResetCallback() {
    this.value = this.getDefaultValue();
  }

  override formStateRestoreCallback(state: string) {
    this.value = state;
  }

  get value() {
    return this.editor?.getValue() ?? this.getInitialValue();
  }

  set value(value: string) {
    const editor = this.editor;

    if (!editor) {
      throw new Error("editor isn't initialized yet");
    }

    editor.setValue(value);
  }
}

declare module 'preact' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace JSX {
    interface EditorElementAttributes extends JSX.HTMLAttributes<HTMLElement> {
      autofocus?: boolean;
      readonly?: boolean;
      required?: boolean;
      disabled?: boolean;
      defaultValue?: string;
      value?: string;
      placeholder?: string;
    }

    interface IntrinsicElements {
      'v-editor': EditorElementAttributes;
    }
  }
}

customElements.define('v-editor', HTMLVEditorElement);
