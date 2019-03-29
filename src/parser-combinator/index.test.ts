import { test } from '~/tester'
import {
  expectStr,
  andThen,
  orElse,
  mapP,
} from './index'

test('matcher expectStr', (assert) => {
  assert.true(expectStr('test')('test', 0).success)
  assert.false(expectStr('test')('te', 0).success)
  assert.false(expectStr('test')('test', 3).success)
  assert.false(expectStr('test')('not ok', 0).success)
})

test('mapP', (assert) => {
  const mapper = mapP(() => ({ kind: 'dummy' }), expectStr('test'))
  assert.false(mapper('te', 0).success)

  const result = mapper('test', 0)
  assert.true(result.success)

  if (result.success) {
    assert.true(result.result.kind === 'dummy')
  }
})

test('combinator andThen', (assert) => {
  const parser = andThen(expectStr('x1'), expectStr('x2'))

  assert.false(parser('0x1x23', 0).success)
  assert.true(parser('0x1x23', 1).success)
})

test('combinator orElse', (assert) => {
  const parser = orElse(expectStr('x1'), expectStr('y'))

  assert.false(parser('0x1y', 0).success)
  assert.true(parser('0x1y', 1).success)
  assert.true(parser('0x1y', 3).success)
})

test('combine andThen and orElse', (assert) => {
  const parser = andThen(expectStr('x1'), orElse(expectStr('2'), expectStr('3')))
  assert.false(parser('x11', 0).success)
  assert.true(parser('x12', 0).success)
  assert.true(parser('x13', 0).success)
})
