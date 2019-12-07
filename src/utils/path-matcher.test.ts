import {
  test,
  assertDeepEqual,
  assertFalse,
} from '~/tester'

import { PathMatcher } from './path-matcher'

test('matches strings', () => {
  const result = PathMatcher.create().string('test').string('param').match('/test/param')
  assertDeepEqual(result, {})
})

test("doesn't match bad strings", () => {
  {
    const result = PathMatcher.create().string('test').string('param').match('/test')
    assertFalse(!!result)
  }

  {
    const result = PathMatcher.create().string('test').string('param').match('/test/param1')
    assertFalse(!!result)
  }
})

test('matches strings and params', () => {
  {
    const result = PathMatcher.create().param('test').string('param').match('/123/param')
    assertDeepEqual(result, { test: '123' })
  }

  {
    const result = PathMatcher.create().string('test').param('param').match('/test/param')
    assertDeepEqual(result, { param: 'param' })
  }
})

test('matches everything', () => {
  {
    const result = PathMatcher.create().param('test').everything().match('/123/param')
    assertDeepEqual(result, { test: '123', everything: ['param'] })
  }

  {
    const result = PathMatcher.create().everything().match('/test/param')
    assertDeepEqual(result, { everything: ['test', 'param'] })
  }

  {
    const result = PathMatcher.create().string('test').everything().match('/test/')
    assertDeepEqual(result, { everything: [''] })
  }

  {
    const result = PathMatcher.create().everything().match('/')
    assertDeepEqual(result, { everything: [''] })
  }
})
