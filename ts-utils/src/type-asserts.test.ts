import {
  test,
  assertTrue,
  assertFalse,
} from '@v/tester'
import {
  isFunction,
  isObject,
} from './type-asserts'

test('isObject', () => {
  assertFalse(isObject(undefined))
  assertTrue(isObject({}))
})

test('isFunction', () => {
  assertTrue(isFunction(() => true))
  // eslint-disable-next-line @typescript-eslint/require-await
  assertTrue(isFunction(async () => true))
  // eslint-disable-next-line prefer-arrow-callback
  assertTrue(isFunction(function testIsFunction() { }))
  // eslint-disable-next-line @typescript-eslint/require-await, prefer-arrow-callback
  assertTrue(isFunction(async function testIsFunction() { return true }))
})
