import { render } from 'preact';
import { FormControlElement } from 'components/Form/FormControlElement';
import { RefInput } from './RefInput';

export class RefClickEvent extends CustomEvent<{ documentId: string }> {
  constructor(public readonly documentId: string) {
    super('RefClick', { detail: { documentId } });
  }
}

export class RefsChangeEvent extends CustomEvent<{ refs: string[] }> {
  constructor(public readonly refs: string[]) {
    super('RefsChange', { detail: { refs } });
  }
}

function parseRefsList(refs: string): string[] {
  return refs
    .replaceAll(',', ' ')
    .split(' ')
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
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

    const onChange = (ids: string[]) => {
      this.value = ids.join(', ');
      this.updateFormValue();
      this.dispatchEvent(new RefsChangeEvent(ids));
    };

    const onRefClick = (documentId: string) => {
      this.dispatchEvent(new RefClickEvent(documentId));
    };

    render(
      <RefInput
        documentTypes={this.documentTypes}
        ids={this.refs}
        multiple={this.multiple}
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

  get documentTypes(): string[] {
    const documentTypes = this.getAttribute('documentTypes');
    if (documentTypes === null) {
      throw new Error('documentTypes attribute is missing');
    }

    return JSON.parse(documentTypes) as string[];
  }

  get multiple() {
    return this.hasAttribute('multiple');
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

  get refs() {
    return parseRefsList(this.value);
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

      documentTypes: string;
      multiple?: boolean;
      onRefClick?: (e: RefClickEvent) => void;
      onRefsChange?: (e: RefsChangeEvent) => void;
    }

    interface IntrinsicElements {
      'v-ref-input': RefInputElementAttributes;
    }
  }
}

customElements.define('v-ref-input', HTMLVRefInputElement);
