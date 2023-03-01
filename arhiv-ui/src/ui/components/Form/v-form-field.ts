import 'element-internals-polyfill';
import { JSONValue } from 'utils';

// This is a helper component that allows to build custom form fields, with validation!
export class HTMLVFormFieldElement extends HTMLElement {
  static get formAssociated() {
    return true;
  }

  static get observedAttributes() {
    return ['required', 'disabled']; // NOTE: readonly not needed since this component doesn't provide UI
  }

  private internals = this.attachInternals();
  private _value: JSONValue = null;

  connectedCallback() {
    this._value = this.getDefaultValue();

    this.updateFormValue();
    this.updateTabIndex();
  }

  attributeChangedCallback() {
    this.updateFormValue();
    this.updateTabIndex();
  }

  formResetCallback() {
    this._value = this.getDefaultValue();
    this.updateFormValue();
  }

  formStateRestoreCallback(state: string) {
    this._value = state;
    this.updateFormValue();
  }

  private updateTabIndex = () => {
    if (this.disabled) {
      this.tabIndex = -1;
    } else {
      // element must be focusable for form validation to work
      this.tabIndex = 0;
    }
  };

  private updateFormValue = () => {
    if (this.disabled) {
      this.internals.setFormValue(null);
      return;
    }

    const value = this._value;

    if (this.required && !value) {
      this.internals.setValidity({ valueMissing: true }, 'Field must not be empty');
    } else {
      this.internals.setValidity({});
    }

    this.internals.setFormValue(JSON.stringify(value));
  };

  private getDefaultValue = () =>
    JSON.parse(this.getAttribute('defaultValue') ?? 'null') as JSONValue;

  get disabled() {
    return this.hasAttribute('disabled');
  }

  get required() {
    return this.hasAttribute('required');
  }

  get value(): JSONValue {
    return this._value;
  }

  set value(value: JSONValue) {
    this._value = value;
    this.updateFormValue();
  }

  get form(): HTMLFormElement | null {
    return this.internals.form;
  }

  get type() {
    return this.localName;
  }

  get name() {
    return this.getAttribute('name');
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
    interface FormFieldElementAttributes extends JSX.HTMLAttributes<HTMLElement> {
      required?: boolean;
      disabled?: boolean;
      defaultValue?: string;
      value?: never;
      name: string;
    }

    interface IntrinsicElements {
      'v-form-field': FormFieldElementAttributes;
    }
  }
}

customElements.define('v-form-field', HTMLVFormFieldElement);
