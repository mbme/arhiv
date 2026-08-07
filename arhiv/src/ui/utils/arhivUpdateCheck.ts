const RELEASE_TAG_PATTERN = /^(\d+)$/;
const BUILD_VERSION_PATTERN = /^(\d+)(?:-\d+-g[0-9a-f]+)?$/;

export const UPDATE_CHECK_INTERVAL_MS = 60 * 60 * 1000;

/**
 * Returns a validated stable Arhiv release tag from the GitHub API response.
 */
export function getLatestArhivReleaseTag(response: unknown): string | undefined {
  if (typeof response !== 'object' || response === null || Array.isArray(response)) {
    return undefined;
  }

  const tagName = (response as Record<string, unknown>).tag_name;
  if (typeof tagName !== 'string' || !RELEASE_TAG_PATTERN.test(tagName)) {
    return undefined;
  }

  return tagName;
}

/**
 * Returns whether a validated stable release is newer than the current Arhiv build.
 */
export function isArhivUpdateAvailable(currentBuild: string, latestReleaseTag: string): boolean {
  const currentRelease = parseReleaseNumber(currentBuild, BUILD_VERSION_PATTERN);
  const latestRelease = parseReleaseNumber(latestReleaseTag, RELEASE_TAG_PATTERN);

  return (
    currentRelease !== undefined && latestRelease !== undefined && latestRelease > currentRelease
  );
}

/**
 * Returns whether the current build may make another update-check request.
 */
export function shouldCheckForArhivUpdates(
  currentBuild: string,
  lastAttemptAtMs: number,
  nowMs: number,
): boolean {
  if (
    parseReleaseNumber(currentBuild, BUILD_VERSION_PATTERN) === undefined ||
    !Number.isFinite(nowMs)
  ) {
    return false;
  }

  return (
    !Number.isFinite(lastAttemptAtMs) ||
    lastAttemptAtMs <= 0 ||
    nowMs < lastAttemptAtMs ||
    nowMs - lastAttemptAtMs >= UPDATE_CHECK_INTERVAL_MS
  );
}

function parseReleaseNumber(value: string, pattern: RegExp): number | undefined {
  const match = pattern.exec(value);
  if (!match) {
    return undefined;
  }

  const releaseNumber = Number(match[1]);
  return Number.isFinite(releaseNumber) ? releaseNumber : undefined;
}
