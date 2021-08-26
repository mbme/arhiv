class VCatalog extends HTMLElement {
  constructor() {
    super();
    // this.innerHTML = '<div>hello <slot></slot></div>';
  }
  connectedCallback() {
    console.log(this.innerHTML);
  }
}

customElements.define('v-catalog', VCatalog);
