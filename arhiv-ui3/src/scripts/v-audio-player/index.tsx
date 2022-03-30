import { render } from 'preact';
import { AudioPlayer } from './audio-player';

export class AudioPlayerElement extends HTMLElement {
  static get observedAttributes() {
    return ['data-artist', 'data-title', 'url', 'autoplay'];
  }

  connectedCallback() {
    this.render();
  }

  attributeChangedCallback() {
    this.render();
  }

  setTrack(artist: string, title: string, url: string) {
    this.setAttribute('data-artist', artist);
    this.setAttribute('data-title', title);
    this.setAttribute('url', url);
  }

  private render() {
    const artist = this.getAttribute('data-artist') || '';
    const title = this.getAttribute('data-title') || '';
    const url = this.getAttribute('url') || '';
    const autoplay = this.hasAttribute('autoplay');

    render(<AudioPlayer artist={artist} title={title} url={url} autoplay={autoplay} />, this);
  }
}

customElements.define('v-audio-player', AudioPlayerElement);
