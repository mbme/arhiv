import {
  test,
  assert,
} from '~/tester'
import {
  expect,
  satisfy,
  regex,
  eof,
  everythingUntil,
} from './matchers'
import {
  isFailure,
} from './parser'
import {
  assertFailure,
  assertSuccess,
} from './test-utils'

test('map', () => {
  const mapper = expect('test').map(() => ({ kind: 'dummy' }))
  assertFailure(mapper.apply('te', 0))

  assertSuccess(mapper.apply('test', 0), (result) => {
    assert.true(result.kind === 'dummy')
  })
})

test('andThen', () => {
  const parser = expect('x1').andThen(expect('x2'))

  assertFailure(parser.apply('0x1x23', 0))
  assertSuccess(parser.apply('0x1x23', 1))
})

test('orElse', () => {
  const parser = expect('y').orElse(expect('x1'))

  assertFailure(parser.apply('0x1y', 0))
  assertSuccess(parser.apply('0x1y', 1))
  assertSuccess(parser.apply('0x1y', 3))
})

test('andThen and orElse', () => {
  const parser = expect('x1').andThen(
    expect('2').orElse(expect('3')),
  )
  assertFailure(parser.apply('x11', 0))
  assertSuccess(parser.apply('x12', 0))
  assertSuccess(parser.apply('x13', 0))
})

test('oneOrMore', () => {
  const parser = expect('x1').oneOrMore()

  assertFailure(parser.apply('x2', 0))
  assertSuccess(parser.apply('x1', 0))

  assertSuccess(parser.apply('x1x1x12', 0), (result) => {
    assert.equal(result.length, 3)
  })
})

test('zeroOrMore', () => {
  const parser = expect('x1').zeroOrMore()

  assertSuccess(parser.apply('x2', 0))
  assertSuccess(parser.apply('x1', 0))
})

test('optional', () => {
  const parser = expect('x1').optional()

  assertSuccess(parser.apply('x2', 0), (result) => {
    assert.equal(result, undefined)
  })

  assertSuccess(parser.apply('x1', 0), (result) => {
    assert.equal(result, 'x1')
  })
})

test('everythingUntil', () => {
  const parser = everythingUntil(expect('x1'))

  assertSuccess(parser.apply('testx1', 0), (result) => {
    assert.equal(result, 'test')
  })

  assertFailure(parser.apply('x2', 0))
})

test('between', () => {
  const parser = expect('test').between(expect('x1'), expect('x1'))

  assertSuccess(parser.apply('x1testx1', 0), (result) => {
    assert.equal(result, 'test')
  })

  assertFailure(parser.apply('x1test', 0))
  assertFailure(parser.apply('x1testx2', 0))
})

test('withLabel', () => {
  const parser = expect('test').withLabel('WORKS')

  const result = parser.apply('te', 0)
  assertFailure(result)
  if (isFailure(result)) {
    assert.true(result.label.includes('WORKS'))
  }
})

test('parseAll', () => {
  const parser = expect('x1')

  assertSuccess(parser.parseAll('x1'))
  assertFailure(parser.parseAll('x1 '))
})

test('eof', () => {
  assertSuccess(eof.apply('test', 4))
  assertFailure(eof.apply('test', 3))
})

test('satisfy', () => {
  const matcher = satisfy((x, pos) => {
    if (/#test/.test(x.substring(pos))) {
      return [true, '#test']
    }

    return [false, 'No match']
  })

  assertSuccess(matcher.apply('#test', 0))
  assertFailure(matcher.apply('#htest', 0))
})

test('expect', () => {
  const parser = expect('test')
  assertSuccess(parser.apply('test', 0))
  assertFailure(parser.apply('te', 0))
  assertFailure(parser.apply('test', 3))
  assertFailure(parser.apply('not ok', 0))
})

test('regex', () => {
  assertSuccess(regex(/^test/).apply('test', 0))

  assertSuccess(regex(/^0*1+/).apply('001', 0), (result) => {
    assert.equal(result, '001')
  })

  assertFailure(regex(/^test/).apply('not test', 0))
})
