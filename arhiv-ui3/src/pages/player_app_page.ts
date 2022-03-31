import { fetchJSON } from '../scripts/utils';
import { AudioPlayerElement } from '../scripts/v-audio-player';

type Attachment = {
  data: {
    blob: string;
  };
};

export function initPlayerApp(rootEl: HTMLElement): void {
  const player = rootEl.querySelector('#player') as AudioPlayerElement;

  const list = rootEl.querySelector<HTMLUListElement>('#track-list');
  if (!list) {
    throw new Error("couldn't find #track-list");
  }

  let activeTrackEl: HTMLElement | undefined = undefined;

  function play(trackEl?: HTMLElement) {
    activeTrackEl?.classList.remove('is-active');

    activeTrackEl = trackEl;
    activeTrackEl?.classList.add('is-active');

    if (!trackEl) {
      player.stop();
      return;
    }

    const trackId = trackEl.dataset.trackId || '';
    const artist = trackEl.dataset.artist || '';
    const title = trackEl.dataset.title || '';

    fetchJSON<Attachment>(`/documents/${trackId}`)
      .then((document) => {
        const blobId = document.data.blob;

        player.setTrack(artist, title, `/blobs/${blobId}`);
      })
      .catch((e) => {
        console.error('Failed to play track %s - %s', artist, title, e);
      });
  }

  const listTrackEls = () => Array.from(list.children) as HTMLElement[];

  player.onNextTrack = () => {
    const trackEls = listTrackEls();

    if (trackEls.length === 0) {
      play(undefined);
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
