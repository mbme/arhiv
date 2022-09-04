import { useEffect, useRef, useState } from 'react';
import { Callback } from '../utils';

export type AudioPlayerState = 'initial' | 'ready' | 'playing' | 'paused';

export type AudioState = {
  audio: HTMLAudioElement;
  currentTimeS: number;
  durationS: number;
  playerState: AudioPlayerState;
  volume: number;
  muted: boolean;
};

const SESSION_STATE_KEY = 'v-audio-player-volume-state';

type AudioVolumeState = {
  volume: number;
  muted: boolean;
};

function getVolumeState(): AudioVolumeState {
  const volumeStateStr = sessionStorage.getItem(SESSION_STATE_KEY);
  return volumeStateStr
    ? (JSON.parse(volumeStateStr) as AudioVolumeState)
    : { volume: 0.5, muted: false };
}

type Options = {
  autoplay: boolean;
  onTrackEnded?: Callback;
};

const DEFAULT_OPTIONS: Options = {
  autoplay: false,
};

export function useAudio(url: string, options: Options = DEFAULT_OPTIONS): AudioState {
  const [currentTimeS, setCurrentTimeS] = useState(0);
  const [duration, setDuration] = useState(Infinity);
  const [playerState, setPlayerState] = useState<AudioPlayerState>('initial');
  const [volume, setVolume] = useState(0);
  const [muted, setMuted] = useState(false);

  const optionsRef = useRef(options);
  optionsRef.current = options;

  const [audio] = useState(() => {
    const audio = new Audio();
    audio.preload = 'metadata';
    audio.autoplay = options.autoplay;

    const volumeState = getVolumeState();

    audio.volume = volumeState.volume;
    audio.muted = volumeState.muted;

    return audio;
  });

  useEffect(() => {
    console.debug('audio: src changed to "%s"', url);
    audio.src = url || '';
    setPlayerState('initial');
  }, [audio, url]);

  useEffect(() => {
    console.debug('audio: autoplay=%s', options.autoplay);
    audio.autoplay = options.autoplay;
  }, [audio, options.autoplay]);

  useEffect(() => {
    setVolume(audio.volume);
    setMuted(audio.muted);

    const onLoadedMetadata = () => {
      setCurrentTimeS(audio.currentTime);
      setDuration(audio.duration);
      setPlayerState((currentState) => (currentState === 'initial' ? 'ready' : currentState));
      console.debug('audio: loaded metadata');
    };
    const onLoadedData = () => {
      console.debug('audio: loaded data');
    };
    const onCanPlay = () => {
      console.debug('audio: can play');
    };
    const onCanPlayThrough = () => {
      console.debug('audio: can play through');
    };
    const onWaiting = () => {
      console.info('audio: waiting');
    };
    const onStalled = () => {
      console.warn('audio: stalled');
    };
    const onPlay = () => {
      setPlayerState('playing');
      console.debug('audio: playing');
    };
    const onPause = () => {
      setPlayerState('paused');
      console.debug('audio: paused');
    };
    const onTimeupdate = () => {
      setCurrentTimeS(audio.currentTime);
    };
    const onDurationChange = () => {
      setDuration(audio.duration);
      console.debug('audio: duration changed');
    };
    const onVolumeChange = () => {
      setVolume(audio.volume);
      setMuted(audio.muted);

      const volumeState: AudioVolumeState = {
        volume: audio.volume,
        muted: audio.muted,
      };
      sessionStorage.setItem(SESSION_STATE_KEY, JSON.stringify(volumeState));
    };
    const onEnded = () => {
      const onTrackEnded = optionsRef.current?.onTrackEnded;

      if (onTrackEnded) {
        onTrackEnded();
      } else {
        // reset playback state on playback ended
        audio.currentTime = 0;
      }

      console.debug('audio: track ended');
    };
    const onError = () => {
      if (!audio.error) {
        console.error('audio: got error event, but there is no error on audio element');
        return;
      }

      // ignore errors due to empty src attribute
      if (audio.error.code === 4) {
        return;
      }

      console.error('audio error:', audio.error);
    };

    // metadata might be loaded before we installed the 'loadedmetadata' event handler
    if (audio.readyState === 1 /* HAVE METADATA */) {
      onLoadedMetadata();
    }

    audio.addEventListener('loadedmetadata', onLoadedMetadata);
    audio.addEventListener('loadeddata', onLoadedData);
    audio.addEventListener('canplay', onCanPlay);
    audio.addEventListener('canplaythrough', onCanPlayThrough);
    audio.addEventListener('waiting', onWaiting);
    audio.addEventListener('stalled', onStalled);
    audio.addEventListener('play', onPlay);
    audio.addEventListener('pause', onPause);
    audio.addEventListener('ended', onEnded);
    audio.addEventListener('timeupdate', onTimeupdate);
    audio.addEventListener('durationchange', onDurationChange);
    audio.addEventListener('volumechange', onVolumeChange);
    audio.addEventListener('error', onError);

    return () => {
      audio.removeEventListener('loadedmetadata', onLoadedMetadata);
      audio.removeEventListener('loadeddata', onLoadedData);
      audio.removeEventListener('canplay', onCanPlay);
      audio.removeEventListener('canplaythrough', onCanPlayThrough);
      audio.removeEventListener('waiting', onWaiting);
      audio.removeEventListener('stalled', onStalled);
      audio.removeEventListener('play', onPlay);
      audio.removeEventListener('pause', onPause);
      audio.removeEventListener('ended', onEnded);
      audio.removeEventListener('timeupdate', onTimeupdate);
      audio.removeEventListener('durationchange', onDurationChange);
      audio.removeEventListener('volumechange', onVolumeChange);
      audio.removeEventListener('error', onError);
    };
  }, [audio]);

  return {
    currentTimeS: Math.round(currentTimeS),
    durationS: Math.round(duration),
    playerState,
    volume,
    muted,
    audio,
  };
}
