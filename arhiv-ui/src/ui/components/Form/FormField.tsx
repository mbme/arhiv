import 'element-internals-polyfill';
import React, { useEffect, useRef } from 'react';
import { JSONValue } from 'utils';
import { useLatestRef } from 'utils/hooks';
import { JSXChildren, JSXRef, mergeRefs } from 'utils/jsx';

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

  protected connectedCallback() {
    this._value = this.getDefaultValue();

    this.updateFormValue();
    this.updateTabIndex();
  }

  protected attributeChangedCallback() {
    this.updateFormValue();
    this.updateTabIndex();
  }

  protected formResetCallback() {
    this._value = this.getDefaultValue();
    this.updateFormValue();
  }

  protected formStateRestoreCallback(state: string) {
    this._value = JSON.parse(state) as JSONValue;
    this.updateFormValue();
  }

  private updateTabIndex() {
    if (this.disabled) {
      this.tabIndex = -1;
    } else {
      // note: element must be focusable for form validation to work
      this.tabIndex = Number.parseInt(this.getAttribute('tabindex') || '0', 10);
    }
  }

  private updateFormValue() {
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
  }

  private getDefaultValue() {
    const value = this.getAttribute('defaultValue') ?? 'null';

    return JSON.parse(value) as JSONValue;
  }

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

  inputValue(value: JSONValue) {
    // TODO deep equality check
    if (this._value === value) {
      return;
    }

    this.value = value;
    this.dispatchEvent(new Event('input', { bubbles: true }));
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

customElements.define('v-form-field', HTMLVFormFieldElement);

type Props = {
  innerRef?: JSXRef<HTMLVFormFieldElement>;
  id?: string;
  className?: string;
  required?: boolean;
  disabled?: boolean;
  hidden?: boolean;
  defaultValue?: JSONValue;
  onFocus?: () => void;
  name: string;
  children?: JSXChildren;
  tabIndex?: number;
};

export function FormField({
  innerRef,
  id,
  className,
  required,
  disabled,
  hidden,
  defaultValue,
  onFocus,
  name,
  children,
  tabIndex,
}: Props) {
  const ref = useRef<HTMLVFormFieldElement>(null);

  const onFocusRef = useLatestRef(onFocus);
  useEffect(() => {
    const el = ref.current;
    if (!el) {
      throw new Error('element is missing');
    }

    const handleFocus = () => onFocusRef.current?.();

    el.addEventListener('focus', handleFocus);

    return () => {
      el.removeEventListener('focus', handleFocus);
    };
  }, [onFocusRef]);

  return React.createElement(
    'v-form-field',
    {
      ref: mergeRefs(ref, innerRef),
      id,
      className,
      required: required || undefined,
      disabled: disabled || undefined,
      hidden: hidden || undefined,
      defaultvalue: defaultValue === undefined ? undefined : JSON.stringify(defaultValue),
      name,
      tabindex: tabIndex,
    },
    children
  );
}
