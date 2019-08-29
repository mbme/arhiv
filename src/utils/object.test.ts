import {
  test,
  asserts,
} from '~/tester'
import {
  merge,
} from './object'

test('merge', () => {
  asserts.equal(merge(null, true), true)
  asserts.deepEqual(merge({}, { x: 1 }), { x: 1 })
  asserts.deepEqual(merge({ x: 0 }, { x: { y: 2 } }), { x: { y: 2 } })
})
