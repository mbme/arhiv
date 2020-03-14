import {
  test,
  assertDeepEqual,
  assertFalse,
} from '@v/tester'

import {
  pathMatcher as pm,
} from './path-matcher'

test('matches strings', () => {
  const result = pm`/test/param`.match('/test/param')
  assertDeepEqual(result, {})
})

test("doesn't match bad strings", () => {
  {
    const result = pm`/test/param`.match('/test')
    assertFalse(!!result)
  }

  {
    const result = pm`/test/param`.match('/test/param1')
    assertFalse(!!result)
  }
})

test('matches strings and params', () => {
  {
    const result = pm`/${'test'}/param`.match('/123/param')
    assertDeepEqual(result, { test: '123' })
  }

  {
    const result = pm`/test/${'param'}`.match('/test/param')
    assertDeepEqual(result, { param: 'param' })
  }

  {
    const result = pm`/test/${'param'}/234`.match('/test//234')
    assertDeepEqual(result, { param: '' })
  }
})

test('matches everything', () => {
  {
    const result = pm`/${'test'}/${'*'}`.match('/123/param')
    assertDeepEqual(result, { test: '123', '*': 'param' })
  }

  {
    const result = pm`/${'*'}`.match('/test/param')
    assertDeepEqual(result, { '*': 'test/param' })
  }

  {
    const result = pm`/test/${'*'}`.match('/test/')
    assertDeepEqual(result, { '*': '' })
  }

  {
    const result = pm`/${'*'}`.match('/')
    assertDeepEqual(result, { '*': '' })
  }
})
