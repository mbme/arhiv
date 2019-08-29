import {
  test,
  asserts,
} from '~/tester'
import {
  createArray,
} from './array'

test('createArray', () => {
  asserts.deepEqual(createArray(3, 0), [0, 0, 0])
  asserts.deepEqual(createArray(3, (i) => i), [0, 1, 2])
})
