/* oxlint-disable typescript/no-floating-promises */

import assert from 'node:assert';
import { describe, it } from 'node:test';
import {
  UPDATE_CHECK_INTERVAL_MS,
  getLatestArhivReleaseTag,
  isArhivUpdateAvailable,
  shouldCheckForArhivUpdates,
} from './arhivUpdateCheck';

describe('getLatestArhivReleaseTag()', () => {
  it('accepts numeric release tags only', () => {
    assert.equal(getLatestArhivReleaseTag({ tag_name: '112' }), '112');
    assert.equal(getLatestArhivReleaseTag({ name: '112' }), undefined);
    assert.equal(getLatestArhivReleaseTag({ tag_name: 'release-112' }), undefined);
    assert.equal(getLatestArhivReleaseTag({ tag_name: 112 }), undefined);
    assert.equal(getLatestArhivReleaseTag(null), undefined);
  });
});

describe('isArhivUpdateAvailable()', () => {
  it('compares release tags with release-derived build versions', () => {
    assert.equal(isArhivUpdateAvailable('111', '112'), true);
    assert.equal(isArhivUpdateAvailable('111-3-gabcdef', '112'), true);
    assert.equal(isArhivUpdateAvailable('112-3-gabcdef', '112'), false);
    assert.equal(isArhivUpdateAvailable('113', '112'), false);
  });

  it('does not compare unrecognized versions', () => {
    assert.equal(isArhivUpdateAvailable('dev-build', '112'), false);
    assert.equal(isArhivUpdateAvailable('112', 'release-113'), false);
  });
});

describe('shouldCheckForArhivUpdates()', () => {
  const now = UPDATE_CHECK_INTERVAL_MS * 2;

  it('checks recognized builds on the first launch and after the interval', () => {
    assert.equal(shouldCheckForArhivUpdates('112', 0, now), true);
    assert.equal(shouldCheckForArhivUpdates('112', now - UPDATE_CHECK_INTERVAL_MS, now), true);
  });

  it('limits checks to one attempt per interval', () => {
    assert.equal(shouldCheckForArhivUpdates('112', now - UPDATE_CHECK_INTERVAL_MS + 1, now), false);
  });

  it('checks again when the clock moved backwards', () => {
    assert.equal(shouldCheckForArhivUpdates('112', now + 1, now), true);
  });

  it('skips unrecognized builds', () => {
    assert.equal(shouldCheckForArhivUpdates('dev-build', 0, now), false);
  });
});
