import { useEffect } from 'react';
import { effect, signal } from '@preact/signals-core';
import { useSignal } from 'utils/hooks';
import { storage } from 'utils/storage';
import { IconButton } from 'components/Button';
import { showToast } from 'components/Toaster';

const KEY = 'LATEST_ARHIV_VERSION';
const $latestVersion = signal(storage.getValue<string>(KEY, ''));
effect(() => {
  storage.setValue(KEY, $latestVersion.value);
});

const LAST_CHECK_KEY = 'LATEST_ARHIV_VERSION_LAST_CHECK';
const $lastCheck = signal(storage.getValue<string>(LAST_CHECK_KEY, '0'));
effect(() => {
  storage.setValue(LAST_CHECK_KEY, $lastCheck.value);
});

export function OutdatedChecker() {
  const latestVersion = useSignal($latestVersion);

  useEffect(() => {
    const lastCheck = Number($lastCheck.value);
    const now = Date.now();

    // max one check per day
    if (now - lastCheck < 24 * 60 * 60 * 1000) {
      return;
    }

    const abortController = new AbortController();
    fetch('https://api.github.com/repos/mbme/typed-v/releases/latest', {
      signal: abortController.signal,
    })
      .then((res) => {
        if (!res.ok) {
          throw new Error(`GitHub API returned ${res.status}`);
        }
        return res.json();
      })
      .then((data) => {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        $latestVersion.value = data.name as string;
        $lastCheck.value = now.toString();
      })
      .catch((error: unknown) => {
        console.error('Failed to fetch latest version', error);
      });

    return () => {
      abortController.abort();
    };
  }, []);

  const currentVersion = window.CONFIG.arhivVersion;
  const isOutdated = latestVersion && latestVersion !== currentVersion;

  if (!isOutdated) {
    return null;
  }

  return (
    <IconButton
      icon="error-triangle"
      title={`New Arhiv version ${latestVersion} is available`}
      className="text-orange-500"
      onClick={() => {
        showToast({
          level: 'warn',
          message: `Current Arhiv version ${currentVersion} is outdated. Latest version is ${latestVersion}`,
        });
      }}
    />
  );
}
