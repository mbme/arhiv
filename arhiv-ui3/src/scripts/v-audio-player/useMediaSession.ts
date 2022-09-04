import { useEffect, useRef } from 'react';
import { Callback } from '../utils';

const getMediaSession = () => {
  // WebView on Android doesn't support media session
  if (!('mediaSession' in navigator)) {
    return undefined;
  }

  return navigator.mediaSession;
};

type Options = {
  artist: string;
  title: string;
  nextTrack?: Callback;
  prevTrack?: Callback;
  onStop?: Callback;
};

const SEEK_OFFSET_S = 7;

export function useMediaSession(audio: HTMLAudioElement, options: Options) {
  const optionsRef = useRef(options);
  optionsRef.current = options;

  useEffect(() => {
    const mediaSession = getMediaSession();
    if (!mediaSession) {
      return;
    }

    mediaSession.setActionHandler('play', () => {
      console.info('media session: play');
      audio.play().catch((e) => {
        console.error('Failed to play', e);
      });
    });

    mediaSession.setActionHandler('pause', () => {
      console.info('media session: pause');
      audio.pause();
    });

    mediaSession.setActionHandler('stop', () => {
      console.info('media session: stop');
      optionsRef.current?.onStop?.();
    });

    mediaSession.setActionHandler('nexttrack', () => {
      console.info('media session: nexttrack');
      optionsRef.current?.nextTrack?.();
    });

    mediaSession.setActionHandler('previoustrack', () => {
      console.info('media session: previoustrack');
      optionsRef.current?.prevTrack?.();
    });

    mediaSession.setActionHandler('seekforward', (e) => {
      console.info('media session: seekforward', e.seekOffset);
      audio.currentTime += e.seekOffset || SEEK_OFFSET_S;
    });

    mediaSession.setActionHandler('seekbackward', (e) => {
      console.info('media session: seekbackward', e.seekOffset);
      const currentTime = audio.currentTime - (e.seekOffset || SEEK_OFFSET_S);

      audio.currentTime = Math.max(currentTime, 0);
    });

    mediaSession.setActionHandler('seekto', (e) => {
      console.info('media session: seekto', e.seekTime, 'fastSeek=', e.fastSeek);
      const seekTime = e.seekTime || 0;

      if (e.fastSeek && 'fastSeek' in audio) {
        audio.fastSeek(seekTime);
      } else {
        audio.currentTime = seekTime;
      }
    });

    const updatePositionState = () => {
      if (!Number.isFinite(audio.duration)) {
        return;
      }

      mediaSession.setPositionState({
        duration: audio.duration,
        playbackRate: audio.playbackRate,
        position: audio.currentTime,
      });
    };

    const onPlay = () => {
      mediaSession.playbackState = 'playing';
    };

    const onPause = () => {
      mediaSession.playbackState = 'paused';
    };

    audio.addEventListener('timeupdate', updatePositionState);
    audio.addEventListener('durationchange', updatePositionState);

    audio.addEventListener('play', onPlay);
    audio.addEventListener('pause', onPause);

    return () => {
      audio.removeEventListener('timeupdate', updatePositionState);
      audio.removeEventListener('durationchange', updatePositionState);

      audio.removeEventListener('play', onPlay);
      audio.removeEventListener('pause', onPause);
    };
  }, [audio]);

  useEffect(() => {
    const mediaSession = getMediaSession();
    if (!mediaSession) {
      return;
    }

    mediaSession.metadata = new MediaMetadata({
      artist: options.artist,
      title: options.title,
    });
  }, [options.artist, options.title]);
}
