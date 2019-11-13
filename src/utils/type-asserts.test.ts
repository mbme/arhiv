import {
  test,
  assertTrue,
  assertFalse,
} from '~/tester'
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
  assertTrue(isFunction(async () => true))
  // tslint:disable-next-line:no-empty
  assertTrue(isFunction(function testIsFunction() { }))
  assertTrue(isFunction(async function testIsFunction() { return true }))
})
