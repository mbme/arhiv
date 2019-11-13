import {
  test,
  assertDeepEqual,
} from '~/tester'
import {
  createArray,
} from './array'

test('createArray', () => {
  assertDeepEqual(createArray(3, 0), [0, 0, 0])
  assertDeepEqual(createArray(3, (i) => i), [0, 1, 2])
})
