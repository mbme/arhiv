import {
  test,
  assertDeepEqual,
} from '@v/tester'
import { Cell } from './cell'
import {
  complete,
  observableToArray,
} from './observable/test-utils'

test('Cell works', async () => {
  const cell = new Cell(0)
  const value$ = cell.value$.take(4)

  const promise = observableToArray(value$)

  cell.value = 1
  cell.value = 1
  cell.value = 2

  const result = await promise

  assertDeepEqual(result, [0, 1, 1, 2, complete])
})

test('Cell can skip duplicate values', async () => {
  const cell = new Cell(0, true)
  const value$ = cell.value$.take(4)

  const promise = observableToArray(value$)

  cell.value = 1
  cell.value = 1
  cell.value = 2
  cell.value = 3

  const result = await promise

  assertDeepEqual(result, [0, 1, 2, 3, complete])
})
