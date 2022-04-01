import { useEffect, useRef, useState } from 'preact/hooks';
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

export function useAudio(url?: string, options: Options = DEFAULT_OPTIONS): AudioState {
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
    audio.src = url || '';
    setPlayerState('initial');
  }, [url]);

  useEffect(() => {
    audio.autoplay = options.autoplay;
  }, [options.autoplay]);

  useEffect(() => {
    setVolume(audio.volume);
    setMuted(audio.muted);

    const onLoadedMetadata = () => {
      setCurrentTimeS(audio.currentTime);
      setDuration(audio.duration);
      setPlayerState((currentState) => (currentState === 'initial' ? 'ready' : currentState));
    };
    const onPlay = () => {
      setPlayerState('playing');
    };
    const onPause = () => {
      setPlayerState('paused');
    };
    const onTimeupdate = () => {
      setCurrentTimeS(audio.currentTime);
    };
    const onDurationChange = () => {
      setDuration(audio.duration);
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
      optionsRef.current?.onTrackEnded?.();
    };

    // metadata might be loaded before we installed the 'loadedmetadata' event handler
    if (audio.readyState === 1 /* HAVE METADATA */) {
      onLoadedMetadata();
    }

    audio.addEventListener('loadedmetadata', onLoadedMetadata);
    audio.addEventListener('play', onPlay);
    audio.addEventListener('pause', onPause);
    audio.addEventListener('ended', onEnded);
    audio.addEventListener('timeupdate', onTimeupdate);
    audio.addEventListener('durationchange', onDurationChange);
    audio.addEventListener('volumechange', onVolumeChange);

    return () => {
      audio.removeEventListener('loadedmetadata', onLoadedMetadata);
      audio.removeEventListener('play', onPlay);
      audio.removeEventListener('pause', onPause);
      audio.removeEventListener('ended', onEnded);
      audio.removeEventListener('timeupdate', onTimeupdate);
      audio.removeEventListener('durationchange', onDurationChange);
      audio.removeEventListener('volumechange', onVolumeChange);
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
