import {
  test,
  assertDeepEqual,
  assertTrue,
  assertFalse,
} from '~/tester'

import { PathMatcher } from './path-matcher'

test('matches strings', () => {
  const result = PathMatcher.create().string('test').string('param').match(['test', 'param'])
  assertTrue(!!result)
  assertDeepEqual(result, {})
})

test("doesn't match bad strings", () => {
  const result = PathMatcher.create().string('test').string('param').match(['test', 'param1'])
  assertFalse(!!result)
})

test('matches strings and params', () => {
  {
    const result = PathMatcher.create().param('test').string('param').match(['123', 'param'])
    assertTrue(!!result)
    assertDeepEqual(result, { test: '123' })
  }

  {
    const result = PathMatcher.create().string('test').param('param').match(['test', 'param'])
    assertTrue(!!result)
    assertDeepEqual(result, { param: 'param' })
  }
})
