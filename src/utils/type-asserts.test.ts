import {
  test,
  asserts,
} from '~/tester'
import {
  isFunction,
  isObject,
} from './type-asserts'

test('isObject', () => {
  asserts.false(isObject(undefined))
  asserts.true(isObject({}))
})

test('isFunction', () => {
  asserts.true(isFunction(() => true))
  asserts.true(isFunction(async () => true))
  // tslint:disable-next-line:no-empty
  asserts.true(isFunction(function testIsFunction() { }))
  asserts.true(isFunction(async function testIsFunction() { return true }))
})
