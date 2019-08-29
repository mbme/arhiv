import { test } from '~/tester'
import {
  merge,
} from './object'

test('merge', (assert) => {
  assert.equal(merge(null, true), true)
  assert.deepEqual(merge({}, { x: 1 }), { x: 1 })
  assert.deepEqual(merge({ x: 0 }, { x: { y: 2 } }), { x: { y: 2 } })
})
