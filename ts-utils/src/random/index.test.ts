import {
  test,
  assertDeepEqual,
} from '@v/tester'
import { shuffle } from './index'

test('shuffle', () => {
  const arr = [1, 2, 3, 4]
  const arr1 = shuffle(arr)

  assertDeepEqual(new Set(arr1), new Set(arr))
})
