import { useEffect, useState } from 'preact/hooks';

export type AudioPlayerState = 'initial' | 'ready' | 'playing' | 'paused';

export type AudioState = {
  audio: HTMLAudioElement;
  currentTimeS: number;
  durationS: number;
  playerState: AudioPlayerState;
  volume: number;
  muted: boolean;
};

export function useAudio(url?: string, autoplay = false): AudioState {
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(Infinity);
  const [playerState, setPlayerState] = useState<AudioPlayerState>('initial');
  const [volume, setVolume] = useState(0);
  const [muted, setMuted] = useState(false);

  const [audio] = useState(() => {
    const audio = new Audio();
    audio.preload = 'metadata';
    audio.autoplay = autoplay;

    return audio;
  });

  useEffect(() => {
    audio.src = url || '';
    setPlayerState('initial');
  }, [url]);

  useEffect(() => {
    audio.autoplay = autoplay;
  }, [autoplay]);

  useEffect(() => {
    setVolume(audio.volume);
    setMuted(audio.muted);

    const onLoadedMetadata = () => {
      setCurrentTime(audio.currentTime);
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
      setCurrentTime(audio.currentTime);
    };
    const onDurationChange = () => {
      setDuration(audio.duration);
    };
    const onVolumeChange = () => {
      setVolume(audio.volume);
      setMuted(audio.muted);
    };

    // metadata might be loaded before we installed the 'loadedmetadata' event handler
    if (audio.readyState === 1 /* HAVE METADATA */) {
      onLoadedMetadata();
    }

    audio.addEventListener('loadedmetadata', onLoadedMetadata);
    audio.addEventListener('play', onPlay);
    audio.addEventListener('pause', onPause);
    audio.addEventListener('timeupdate', onTimeupdate);
    audio.addEventListener('durationchange', onDurationChange);
    audio.addEventListener('volumechange', onVolumeChange);

    return () => {
      audio.removeEventListener('loadedmetadata', onLoadedMetadata);
      audio.removeEventListener('play', onPlay);
      audio.removeEventListener('pause', onPause);
      audio.removeEventListener('timeupdate', onTimeupdate);
      audio.removeEventListener('durationchange', onDurationChange);
      audio.removeEventListener('volumechange', onVolumeChange);
    };
  }, [audio]);

  return {
    currentTimeS: Math.round(currentTime),
    durationS: Math.round(duration),
    playerState,
    volume,
    muted,
    audio,
  };
}
