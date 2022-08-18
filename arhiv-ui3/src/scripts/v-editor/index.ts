import { initEditor, EditorView } from './editor';

export class EditorElement extends HTMLElement {
  static get observedAttributes() {
    return ['for', 'autofocus'];
  }

  private textarea?: HTMLTextAreaElement;
  private editor?: EditorView;

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

    this.textarea = textarea;
    const editor = initEditor(this, textarea.value, () => {
      textarea.value = editor.state.doc.toString();
    });
    this.editor = editor;

    textarea.setAttribute('hidden', 'true');

    if (this.hasAttribute('autofocus')) {
      this.editor.focus();
    }

    for (const label of textarea.labels) {
      label.addEventListener('click', this.onLabelClick);
    }
  }

  disconnectedCallback() {
    for (const label of this.textarea?.labels || []) {
      label.removeEventListener('click', this.onLabelClick);
    }
  }

  private onLabelClick = () => {
    this.editor?.focus();
  };
}

customElements.define('v-editor', EditorElement);
