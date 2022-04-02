import { initEditor } from './editor';

export class EditorElement extends HTMLElement {
  static get observedAttributes() {
    return ['for', 'autofocus'];
  }

  connectedCallback() {
    const textareaId = this.getAttribute('for');

    if (!textareaId) {
      throw new Error(`editor is missing a mandatory "for" attribute`);
    }

    const textarea = document.getElementById(textareaId);
    if (!textarea) {
      throw new Error(`can't find textarea using selector '${textareaId}'`);
    }

    if (!(textarea instanceof HTMLTextAreaElement)) {
      throw new Error(`'for' attribute must point to textarea`);
    }

    const editor = initEditor(textarea, this);

    if (this.hasAttribute('autofocus')) {
      editor.focus();
    }
  }
}

customElements.define('v-editor', EditorElement);
