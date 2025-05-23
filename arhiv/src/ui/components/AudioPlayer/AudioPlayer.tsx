import { Callback } from '../../utils';
import { Icon } from '../Icon';
import { useAudio } from './useAudio';
import { useMediaSession } from './useMediaSession';

function formatTime(timeS: number): string {
  if (timeS === Infinity) {
    return '∞';
  }

  const date = new Date(timeS * 1000).toISOString();

  if (timeS < 3600) {
    return date.slice(14, 19);
  }

  return date.slice(11, 19);
}

type Props = {
  title: string;
  artist: string;
  url: string;
  autoplay?: boolean;
  nextTrack?: Callback;
  prevTrack?: Callback;
  onStop?: Callback;
  onTrackEnded?: Callback;
};

export function AudioPlayer({
  title,
  artist,
  url,
  autoplay = false,
  nextTrack,
  prevTrack,
  onStop,
  onTrackEnded,
}: Props) {
  const {
    currentTimeS, //
    durationS,
    playerState,
    volume,
    muted,
    audio,
  } = useAudio(url, {
    autoplay,
    onTrackEnded,
  });

  useMediaSession(audio, {
    artist,
    title,
    nextTrack,
    prevTrack,
    onStop,
  });

  const play = () => {
    audio.play().catch((e: unknown) => {
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

    console.info('player: seeking to', seekTime);

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/fastSeek
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    if (audio.fastSeek) {
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
    <div className="px-4 py-3 audio-player">
      <div
        className="flex items-center gap-3 text-gray-900 cursor-default mb-2 pl-2"
        hidden={!artist && !title}
      >
        <span className="text-xl font-semibold">{artist}</span>
        <span className="text-xl">{title && <>&ndash;</>}</span>
        <span className="text-lg">{title}</span>
      </div>

      <div className="flex justify-between mb-1">
        <div></div>
        <div className="flex gap-6">
          {prevTrack && (
            <button type="button" title="Previous track" onClick={prevTrack}>
              <Icon className="h-10 w-10" variant="skip-previous" />
            </button>
          )}

          <button
            type="button"
            onClick={isPlaying ? pause : play}
            title={isPlaying ? 'Pause' : 'Play'}
            disabled={playerState === 'initial'}
          >
            <Icon className="h-14 w-14" variant={isPlaying ? 'pause' : 'play'} />
          </button>

          {nextTrack && (
            <button type="button" title="Next track" onClick={nextTrack}>
              <Icon className="h-10 w-10" variant="skip-next" />
            </button>
          )}
        </div>

        <div className="flex gap-4 items-center">
          <button
            type="button"
            title={muted ? 'Umute volume' : 'Mute volume'}
            className="text-gray-500"
            onClick={toggleMute}
          >
            <Icon className="h-7 w-7" variant={muted ? 'volume-off' : 'volume-high'} />
          </button>

          <input
            className="w-20"
            type="range"
            min="0"
            max="100"
            step="1"
            value={Math.round(volume * 100)}
            onInput={(e) => {
              onVolumeRangeInput(e.currentTarget);
            }}
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
          onInput={(e) => {
            onDurationRangeInput(e.currentTarget);
          }}
        />

        <output title="Duration" className="cursor-default min-w-[3rem] text-center">
          {formatTime(durationS)}
        </output>
      </div>
    </div>
  );
}
