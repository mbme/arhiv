import { test } from '~/tester'
import {
  createArray,
} from './array'

test('createArray', (assert) => {
  assert.deepEqual(createArray(3, 0), [0, 0, 0])
  assert.deepEqual(createArray(3, (i) => i), [0, 1, 2])
})
