import {
  test,
  assert,
} from '~/tester'
import {
  isFunction,
  isObject,
} from './type-asserts'

test('isObject', () => {
  assert.false(isObject(undefined))
  assert.true(isObject({}))
})

test('isFunction', () => {
  assert.true(isFunction(() => true))
  assert.true(isFunction(async () => true))
  // tslint:disable-next-line:no-empty
  assert.true(isFunction(function testIsFunction() { }))
  assert.true(isFunction(async function testIsFunction() { return true }))
})
