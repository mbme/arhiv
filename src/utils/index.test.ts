import { test } from '~/tester'
import {
  createArray,
  isFunction,
  isObject,
  merge,
} from './index'

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

test('createArray', (assert) => {
  assert.deepEqual(createArray(3, 0), [0, 0, 0])
  assert.deepEqual(createArray(3, (i) => i), [0, 1, 2])
})

test('merge', (assert) => {
  assert.equal(merge(null, true), true)
  assert.deepEqual(merge({}, { x: 1 }), { x: 1 })
  assert.deepEqual(merge({ x: 0 }, { x: { y: 2 } }), { x: { y: 2 } })
})
