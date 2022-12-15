import { render } from 'preact';
import { FormControlElement } from 'components/Form/FormControlElement';
import { RefInput } from './RefInput';

export class RefClickEvent extends CustomEvent<{ documentId: string }> {
  constructor(public readonly documentId: string) {
    super('RefClick', { detail: { documentId } });
  }
}

export class HTMLVRefInputElement extends FormControlElement {
  static get observedAttributes() {
    return ['required', 'readonly', 'disabled'];
  }

  override connectedCallback() {
    this.updateFormValue();
    this.render();
  }

  override attributeChangedCallback() {
    this.render();
    this.updateFormValue();
  }

  private _value?: string;

  private render = () => {
    if (this.disabled) {
      this.tabIndex = -1;
    } else {
      // element must be focusable for form validation to work
      this.tabIndex = 0;
    }

    const onChange = (documentId?: string) => {
      this.value = documentId ?? '';
      this.updateFormValue();
    };

    const onRefClick = (documentId: string) => {
      this.dispatchEvent(new RefClickEvent(documentId));
    };

    render(
      <RefInput
        documentType={this.documentType}
        documentId={this.value || undefined}
        readonly={this.readonly}
        disabled={this.disabled}
        onChange={onChange}
        onRefClick={onRefClick}
      />,
      this
    );
  };

  private updateFormValue = () => {
    if (this.disabled) {
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

  get documentType() {
    const documentType = this.getAttribute('documentType');
    if (documentType === null) {
      throw new Error('documentType attribute is missing');
    }

    return documentType;
  }

  get disabled() {
    return this.hasAttribute('disabled');
  }

  get readonly() {
    return this.hasAttribute('readonly');
  }

  get required() {
    return this.hasAttribute('required');
  }

  override formResetCallback() {
    this.value = this.getDefaultValue();
    this.updateFormValue();
  }

  override formStateRestoreCallback(state: string) {
    this.value = state;
  }

  get value() {
    return this._value ?? this.getInitialValue();
  }

  set value(value: string) {
    this._value = value;
    this.render();
  }
}

declare module 'preact' {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace JSX {
    interface RefInputElementAttributes extends JSX.HTMLAttributes<HTMLElement> {
      readonly?: boolean;
      required?: boolean;
      disabled?: boolean;
      defaultValue?: string;
      value?: string;

      documentType: string;
      onRefClick?: (e: RefClickEvent) => void;
    }

    interface IntrinsicElements {
      'v-ref-input': RefInputElementAttributes;
    }
  }
}

customElements.define('v-ref-input', HTMLVRefInputElement);
