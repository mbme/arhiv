import { test } from '~/tester'
import {
  expect,
  satisfy,
  regex,
  eof,
  everythingUntil,
  between,
} from './index'

test('map', (assert) => {
  const mapper = expect('test').map(() => ({ kind: 'dummy' }))
  assert.false(mapper.apply('te', 0).success)

  const result = mapper.apply('test', 0)
  assert.true(result.success)

  if (result.success) {
    assert.true(result.result.kind === 'dummy')
  }
})

test('andThen', (assert) => {
  const parser = expect('x1').andThen(expect('x2'))

  assert.false(parser.apply('0x1x23', 0).success)
  assert.true(parser.apply('0x1x23', 1).success)
})

test('orElse', (assert) => {
  const parser = expect('y').orElse(expect('x1'))

  assert.false(parser.apply('0x1y', 0).success)
  assert.true(parser.apply('0x1y', 1).success)
  assert.true(parser.apply('0x1y', 3).success)
})

test('andThen and orElse', (assert) => {
  const parser = expect('x1').andThen(
    expect('2').orElse(expect('3')),
  )
  assert.false(parser.apply('x11', 0).success)
  assert.true(parser.apply('x12', 0).success)
  assert.true(parser.apply('x13', 0).success)
})

test('oneOrMore', (assert) => {
  const parser = expect('x1').oneOrMore()

  assert.false(parser.apply('x2', 0).success)
  assert.true(parser.apply('x1', 0).success)

  const result = parser.apply('x1x1x12', 0)
  assert.true(result.success)
  if (result.success) {
    assert.equal(result.result.length, 3)
  }
})

test('zeroOrMore', (assert) => {
  const parser = expect('x1').zeroOrMore()

  assert.true(parser.apply('x2', 0).success)
  assert.true(parser.apply('x1', 0).success)
})

test('optional', (assert) => {
  const parser = expect('x1').optional()

  {
    const result = parser.apply('x2', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result.length, 0)
    }
  }
  {
    const result = parser.apply('x1', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result.length, 1)
    }
  }
})

test('everythingUntil', (assert) => {
  const parser = everythingUntil(expect('x1'))

  {
    const result = parser.apply('testx1', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result, 'test')
    }
  }

  assert.false(parser.apply('x2', 0).success)
})

test('between', (assert) => {
  const parser = between(expect('x1'), expect('x1'))

  {
    const result = parser.apply('x1testx1', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result, 'test')
    }
  }

  assert.false(parser.apply('x1test', 0).success)
  assert.false(parser.apply('x1testx2', 0).success)
})

test('withLabel', (assert) => {
  const parser = expect('test').withLabel('WORKS')

  const result = parser.apply('te', 0)
  assert.false(result.success)
  if (!result.success) {
    assert.equal(result.label, 'WORKS')
  }
})

test('parseAll', (assert) => {
  const parser = expect('x1')

  assert.true(parser.parseAll('x1').success)
  assert.false(parser.parseAll('x1 ').success)
})

test('eof', (assert) => {
  assert.true(eof.apply('test', 4).success)
  assert.false(eof.apply('test', 3).success)
})

test('satisfy', (assert) => {
  const matcher = satisfy((x: string) => {
    if (/#test/.test(x)) {
      return [true, '#test']
    }

    return [false, 'No match']
  })

  assert.true(matcher.apply('#test', 0).success)
  assert.false(matcher.apply('#htest', 0).success)
})

test('expect', (assert) => {
  const parser = expect('test')
  assert.true(parser.apply('test', 0).success)
  assert.false(parser.apply('te', 0).success)
  assert.false(parser.apply('test', 3).success)
  assert.false(parser.apply('not ok', 0).success)
})

test('regex', (assert) => {
  assert.true(regex(/^test/).apply('test', 0).success)

  const result = regex(/^0*1+/).apply('001', 0)
  assert.true(result.success)
  if (result.success) {
    assert.equal(result.result, '001')
  }

  assert.false(regex(/^test/).apply('not test', 0).success)
})
