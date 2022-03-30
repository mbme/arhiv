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

  list.addEventListener('click', (e) => {
    if (!e.target) {
      return;
    }

    const target = e.target as HTMLElement;

    if (target.matches('a')) {
      return;
    }

    const li: HTMLElement | null = target.closest('#track-list > li');
    if (!li) {
      return;
    }

    const trackId = li.dataset.trackId || '';

    const artist = li.dataset.artist || '';
    const title = li.dataset.title || '';

    fetchJSON<Attachment>(`/documents/${trackId}`)
      .then((document) => {
        const blobId = document.data.blob;

        player.setTrack(artist, title, `/blobs/${blobId}`);
      })
      .catch((e) => {
        console.error('Failed to play track %s - %s', artist, title, e);
      });
  });
}
