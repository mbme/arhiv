import { render } from 'preact';
import { Callback } from '../utils';
import { AudioPlayer } from './audio-player';

export class AudioPlayerElement extends HTMLElement {
  static get observedAttributes() {
    return ['data-artist', 'data-title', 'url', 'autoplay'];
  }

  private nextTrack?: Callback;
  private prevTrack?: Callback;
  private trackEnded?: Callback;

  set onNextTrack(nextTrack: Callback | undefined) {
    this.nextTrack = nextTrack;
    this.render();
  }

  get onNextTrack() {
    return this.nextTrack;
  }

  set onPrevTrack(prevTrack: Callback | undefined) {
    this.prevTrack = prevTrack;
    this.render();
  }

  get onPrevTrack() {
    return this.prevTrack;
  }

  set onTrackEnded(ended: Callback | undefined) {
    this.trackEnded = ended;
    this.render();
  }

  get onTrackEnded() {
    return this.trackEnded;
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

  stop = () => {
    this.setAttribute('data-artist', '');
    this.setAttribute('data-title', '');
    this.setAttribute('url', '');
  };

  private render() {
    const artist = this.getAttribute('data-artist') || '';
    const title = this.getAttribute('data-title') || '';
    const url = this.getAttribute('url') || '';
    const autoplay = this.hasAttribute('autoplay');

    render(
      <AudioPlayer
        artist={artist}
        title={title}
        url={url}
        autoplay={autoplay}
        nextTrack={this.nextTrack}
        prevTrack={this.prevTrack}
        onStop={this.stop}
        onTrackEnded={this.trackEnded}
      />,
      this
    );
  }
}

customElements.define('v-audio-player', AudioPlayerElement);
