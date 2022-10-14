import 'element-internals-polyfill';
import type { ElementInternals } from 'element-internals-polyfill/dist/element-internals';

/* eslint-disable @typescript-eslint/no-empty-function */

export abstract class FormControlElement extends HTMLElement {
  protected internals: ElementInternals;

  static get formAssociated() {
    return true;
  }

  constructor() {
    super();

    this.internals = this.attachInternals();
  }

  connectedCallback() {}
  disconnectedCallback() {}
  adoptedCallback() {}
  attributeChangedCallback(_name: string, _oldValue: string | null, _newValue: string | null) {}

  formAssociatedCallback(_form: HTMLFormElement | null) {}
  formDisabledCallback(_isDisabled: boolean) {}
  formResetCallback() {}
  formStateRestoreCallback(_state: string, _mode: 'restore' | 'autocomplete') {}

  get form(): HTMLFormElement | null {
    return this.internals.form;
  }

  get type() {
    return this.localName;
  }

  override get name() {
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
