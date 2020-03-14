import {
  test,
  assertEqual,
  assertDeepEqual,
} from '@v/tester'
import {
  merge,
} from './object'

test('merge', () => {
  assertEqual(merge(null, true), true)
  assertDeepEqual(merge({}, { x: 1 }), { x: 1 })
  assertDeepEqual(merge({ x: 0 }, { x: { y: 2 } }), { x: { y: 2 } })
})
