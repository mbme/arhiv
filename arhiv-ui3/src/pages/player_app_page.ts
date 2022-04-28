import { pickRandomElement } from '../scripts/utils';
import { AudioPlayerElement } from '../scripts/v-audio-player';

export function initPlayerApp(rootEl: HTMLElement): void {
  const player = rootEl.querySelector('#player') as AudioPlayerElement;
  const shuffleTracks = rootEl.querySelector('#shuffle-tracks') as HTMLInputElement;

  const list = rootEl.querySelector<HTMLUListElement>('#track-list');
  if (!list) {
    throw new Error("couldn't find #track-list");
  }

  let activeTrackEl: HTMLElement | undefined = undefined;

  function play(trackEl?: HTMLElement) {
    activeTrackEl?.toggleAttribute('data-is-active', false);

    activeTrackEl = trackEl;
    if (activeTrackEl) {
      activeTrackEl.toggleAttribute('data-is-active', true);
      activeTrackEl.scrollIntoView({
        block: 'center',
      });
    }

    if (!trackEl) {
      player.stop();
      return;
    }

    const blobId = trackEl.dataset.blobId || '';
    const artist = trackEl.dataset.artist || '';
    const title = trackEl.dataset.title || '';

    console.info('Play track %s - %s', artist, title);

    player.setTrack(artist, title, `/blobs/${blobId}`);
  }

  const listTrackEls = () => Array.from(list.children) as HTMLElement[];

  player.onNextTrack = () => {
    const trackEls = listTrackEls();

    if (trackEls.length === 0) {
      play(undefined);
      return;
    }

    if (shuffleTracks.checked) {
      play(pickRandomElement(trackEls));
      return;
    }

    const pos = activeTrackEl ? trackEls.indexOf(activeTrackEl) : -1;

    let nextPos = pos + 1;
    if (nextPos >= trackEls.length) {
      nextPos = 0;
    }

    play(trackEls[nextPos]);
  };

  player.onPrevTrack = () => {
    const trackEls = listTrackEls();

    if (trackEls.length === 0) {
      play(undefined);
      return;
    }

    const pos = activeTrackEl ? trackEls.indexOf(activeTrackEl) : -1;

    let prevPos = pos - 1;
    if (prevPos < 0) {
      prevPos = trackEls.length - 1;
    }

    play(trackEls[prevPos]);
  };

  player.onTrackEnded = () => {
    const trackEls = listTrackEls();

    if (trackEls.length === 0) {
      play(undefined);
      return;
    }

    const pos = activeTrackEl ? trackEls.indexOf(activeTrackEl) : -1;
    if (pos === trackEls.length - 1) {
      play(undefined);
      return;
    }

    const nextPos = pos + 1;

    play(trackEls[nextPos]);
  };

  list.addEventListener('click', (e) => {
    if (!e.target) {
      return;
    }

    const target = e.target as HTMLElement;

    if (target.matches('a')) {
      return;
    }

    const trackEl: HTMLElement | null = target.closest('#track-list > li');

    play(trackEl || undefined);
  });
}
