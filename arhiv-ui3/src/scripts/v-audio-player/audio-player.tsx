import { Callback } from '../utils';
import { useAudio } from './useAudio';
import { useMediaSession } from './useMediaSession';

function formatTime(timeS: number): string {
  if (timeS === Infinity) {
    return '∞';
  }

  const date = new Date(timeS * 1000).toISOString();

  if (timeS < 3600) {
    return date.substr(14, 5);
  }

  return date.substr(11, 8);
}

type Props = {
  title?: string;
  artist?: string;
  url?: string;
  autoplay?: boolean;
  nextTrack?: Callback;
  prevTrack?: Callback;
  onStop?: Callback;
};

export function AudioPlayer({ title, artist, url, autoplay, nextTrack, prevTrack, onStop }: Props) {
  const {
    currentTimeS, //
    durationS,
    playerState,
    volume,
    muted,
    audio,
  } = useAudio(url, autoplay);

  useMediaSession(audio, {
    artist,
    title,
    nextTrack,
    prevTrack,
    onStop,
  });

  const play = () => {
    audio.play().catch((e) => {
      console.error('Failed to play', e);
    });
  };

  const pause = () => {
    audio.pause();
  };

  const toggleMute = () => {
    audio.muted = !muted;
  };

  const onDurationRangeInput = (input: HTMLInputElement) => {
    const seekTime = Number.parseInt(input.value, 10);

    if ('fastSeek' in audio) {
      audio.fastSeek(seekTime);
    } else {
      audio.currentTime = seekTime;
    }
  };

  const onVolumeRangeInput = (input: HTMLInputElement) => {
    const volume = Number.parseInt(input.value, 10) / 100;

    audio.volume = volume;
    audio.muted = false;
  };

  const isPlaying = playerState === 'playing';

  return (
    <div className="px-6 py-4">
      <div className="flex items-center gap-3 text-gray-900 cursor-pointer mb-2">
        <span className="text-xl font-semibold">{artist}</span>
        <span className="text-xl">{title ? '-' : <>&nbsp;</>}</span>
        <span className="text-lg">{title}</span>
      </div>

      <div className="flex justify-between mb-1">
        <div></div>
        <div className="flex gap-6 text-gray-600">
          {prevTrack && (
            <button type="button" title="Previous track" onClick={prevTrack}>
              <svg className="h-10 w-10">
                <use xlinkHref="#icon-rewind" />
              </svg>
            </button>
          )}

          <button
            type="button"
            onClick={isPlaying ? pause : play}
            title={isPlaying ? 'Pause' : 'Play'}
            disabled={playerState === 'initial'}
          >
            <svg className="h-14 w-14">
              <use xlinkHref={isPlaying ? '#icon-pause' : '#icon-play'} />
            </svg>
          </button>

          {nextTrack && (
            <button type="button" title="Next track" onClick={nextTrack}>
              <svg className="h-10 w-10">
                <use xlinkHref="#icon-fast-forward" />
              </svg>
            </button>
          )}
        </div>

        <div className="flex gap-4 items-center">
          <button type="button" title={muted ? 'Umute volume' : 'Mute volume'}>
            <svg className="h-6 w-6 text-gray-500" onClick={toggleMute}>
              <use xlinkHref={muted ? '#icon-volume-off' : '#icon-volume-up'} />
            </svg>
          </button>

          <input
            className="w-20"
            type="range"
            min="0"
            max="100"
            step="1"
            value={Math.round(volume * 100)}
            onInput={(e) => onVolumeRangeInput(e.target as HTMLInputElement)}
          />
        </div>
      </div>

      <div className="flex items-center gap-2 text-sm text-gray-600 mb-1">
        <output title="Current time" className="cursor-default min-w-[3rem] text-center">
          {formatTime(currentTimeS)}
        </output>

        <input
          className="w-full"
          type="range"
          disabled={playerState === 'initial' || durationS === Infinity}
          min="0"
          max={durationS}
          step="1"
          value={currentTimeS}
          onInput={(e) => onDurationRangeInput(e.target as HTMLInputElement)}
        />

        <output title="Duration" className="cursor-default min-w-[3rem] text-center">
          {formatTime(durationS)}
        </output>
      </div>
    </div>
  );
}
