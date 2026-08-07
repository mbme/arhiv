import { useEffect } from 'react';
import { effect, signal } from '@preact/signals-core';
import { useSignal } from '../../utils/hooks';
import { storage } from '../../utils/storage';
import { IconButton } from '../../components/Button';
import { showToast } from '../../components/Toaster';
import {
  getLatestArhivReleaseTag,
  isArhivUpdateAvailable,
  shouldCheckForArhivUpdates,
} from '../../utils/arhivUpdateCheck';

const LATEST_RELEASE_TAG_KEY = 'LATEST_ARHIV_RELEASE_TAG';
// oxlint-disable-next-line typescript/no-unnecessary-type-arguments
const $latestReleaseTag = signal(storage.getValue<string>(LATEST_RELEASE_TAG_KEY, ''));
effect(() => {
  storage.setValue(LATEST_RELEASE_TAG_KEY, $latestReleaseTag.value);
});

const LAST_CHECK_ATTEMPT_KEY = 'LATEST_ARHIV_RELEASE_LAST_CHECK_ATTEMPT';
// oxlint-disable-next-line typescript/no-unnecessary-type-arguments
const $lastCheckAttempt = signal(storage.getValue<number>(LAST_CHECK_ATTEMPT_KEY, 0));
effect(() => {
  storage.setValue(LAST_CHECK_ATTEMPT_KEY, $lastCheckAttempt.value);
});

export function OutdatedChecker() {
  const latestReleaseTag = useSignal($latestReleaseTag);
  const currentVersion = window.CONFIG.arhivVersion;

  useEffect(() => {
    const now = Date.now();
    if (!shouldCheckForArhivUpdates(currentVersion, $lastCheckAttempt.value, now)) {
      return;
    }

    $lastCheckAttempt.value = now;

    const abortController = new AbortController();
    fetch('https://api.github.com/repos/mbme/arhiv/releases/latest', {
      signal: abortController.signal,
    })
      .then((res) => {
        if (!res.ok) {
          throw new Error(`GitHub API returned ${res.status}`);
        }
        return res.json();
      })
      .then((data) => {
        const latestReleaseTag = getLatestArhivReleaseTag(data);
        if (!latestReleaseTag) {
          throw new Error('GitHub API response has no valid Arhiv release tag');
        }

        $latestReleaseTag.value = latestReleaseTag;
      })
      .catch((error: unknown) => {
        if (!abortController.signal.aborted) {
          console.error('Failed to fetch latest Arhiv release', error);
        }
      });

    return () => {
      abortController.abort();
    };
  }, [currentVersion]);

  if (!isArhivUpdateAvailable(currentVersion, latestReleaseTag)) {
    return null;
  }

  return (
    <IconButton
      icon="error-triangle"
      title={`Update available: Arhiv ${latestReleaseTag}`}
      className="text-orange-500"
      onClick={() => {
        showToast({
          level: 'warn',
          message: `Update available: Arhiv ${latestReleaseTag}. Update through your package manager or download a new release.`,
        });
      }}
    />
  );
}
