import { test } from '~/tester'
import {
  isFunction,
  isObject,
} from './type-asserts'

test('isObject', (assert) => {
  assert.equal(isObject(undefined), false)
  assert.equal(isObject({}), true)
})

test('isFunction', (assert) => {
  assert.equal(isFunction(() => true), true)
  assert.equal(isFunction(async () => true), true)
  // tslint:disable-next-line:no-empty
  assert.equal(isFunction(function testIsFunction() { }), true)
  assert.equal(isFunction(async function testIsFunction() { return true }), true)
})
