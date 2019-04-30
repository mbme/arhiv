import { test } from '~/tester'
import {
  expect,
  andThen,
  orElse,
  mapP,
  oneOrMore,
  zeroOrMore,
  optional,
  setLabel,
  satisfy,
  regex,
  eof,
  everythingUntil,
  between,
  parse,
} from './index'

test('mapP', (assert) => {
  const mapper = mapP(() => ({ kind: 'dummy' }), expect('test'))
  assert.false(mapper('te', 0).success)

  const result = mapper('test', 0)
  assert.true(result.success)

  if (result.success) {
    assert.true(result.result.kind === 'dummy')
  }
})

test('andThen', (assert) => {
  const parser = andThen(expect('x1'), expect('x2'))

  assert.false(parser('0x1x23', 0).success)
  assert.true(parser('0x1x23', 1).success)
})

test('orElse', (assert) => {
  const parser = orElse(expect('x1'), expect('y'))

  assert.false(parser('0x1y', 0).success)
  assert.true(parser('0x1y', 1).success)
  assert.true(parser('0x1y', 3).success)
})

test('andThen and orElse', (assert) => {
  const parser = andThen(expect('x1'), orElse(expect('2'), expect('3')))
  assert.false(parser('x11', 0).success)
  assert.true(parser('x12', 0).success)
  assert.true(parser('x13', 0).success)
})

test('oneOrMore', (assert) => {
  const parser = oneOrMore(expect('x1'))

  assert.false(parser('x2', 0).success)
  assert.true(parser('x1', 0).success)

  const result = parser('x1x1x12', 0)
  assert.true(result.success)
  if (result.success) {
    assert.equal(result.result.length, 3)
  }
})

test('zeroOrMore', (assert) => {
  const parser = zeroOrMore(expect('x1'))

  assert.true(parser('x2', 0).success)
  assert.true(parser('x1', 0).success)
})

test('optional', (assert) => {
  const parser = optional(expect('x1'))

  {
    const result = parser('x2', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result.length, 0)
    }
  }
  {
    const result = parser('x1', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result.length, 1)
    }
  }
})

test('everythingUntil', (assert) => {
  const parser = everythingUntil(expect('x1'))

  {
    const result = parser('testx1', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result, 'test')
    }
  }

  assert.false(parser('x2', 0).success)
})

test('between', (assert) => {
  const parser = between(expect('x1'), expect('x1'))

  {
    const result = parser('x1testx1', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result, 'test')
    }
  }

  assert.false(parser('x1test', 0).success)
  assert.false(parser('x1testx2', 0).success)
})

test('setLabel', (assert) => {
  const parser = setLabel(expect('test'), 'WORKS')

  const result = parser('te', 0)
  assert.false(result.success)
  if (!result.success) {
    assert.equal(result.label, 'WORKS')
  }
})

test('parse', (assert) => {
  const parser = expect('x1')

  assert.true(parse(parser, 'x1').success)
  assert.false(parse(parser, 'x1 ').success)
})

test('eof', (assert) => {
  assert.true(eof('test', 4).success)
  assert.false(eof('test', 3).success)
})

test('satisfy', (assert) => {
  const matcher = satisfy((x: string) => {
    if (/#test/.test(x)) {
      return [true, '#test']
    }

    return [false, 'No match']
  })

  assert.true(matcher('#test', 0).success)
  assert.false(matcher('#htest', 0).success)
})

test('expect', (assert) => {
  assert.true(expect('test')('test', 0).success)
  assert.false(expect('test')('te', 0).success)
  assert.false(expect('test')('test', 3).success)
  assert.false(expect('test')('not ok', 0).success)
})

test('regex', (assert) => {
  assert.true(regex(/^test/)('test', 0).success)

  const result = regex(/^0*1+/)('001', 0)
  assert.true(result.success)
  if (result.success) {
    assert.equal(result.result, '001')
  }

  assert.false(regex(/^test/)('not test', 0).success)
})
