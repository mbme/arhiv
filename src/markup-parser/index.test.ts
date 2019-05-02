import { test } from '~/tester'
import {
  newlines,
  paragraph,
  mono,
  bold,
  strikethrough,
  header,
} from './index'

test('inline', (assert) => {
  {
    const result = mono.parseAll('`test`')
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result.type, 'Mono')
      assert.equal(result.result.value, 'test')
    }

    assert.false(mono.parseAll('`te\nst`').success)
  }

  assert.true(bold.parseAll('*test*').success)
  assert.true(strikethrough.parseAll('~test~').success)
})

test('header', (assert) => {
  {
    const result = header.parseAll('# header')
    assert.true(result.success)
    if (result.success) {
      const { type, value: [level, str] } = result.result
      assert.equal(type, 'Header')
      assert.equal(level, 1)
      assert.equal(str, 'header')
    }
  }

  {
    const result = header.apply('test\n## header\ntest', 4)
    assert.true(result.success)
    if (result.success) {
      const { value: [level, str] } = result.result
      assert.equal(level, 2)
      assert.equal(str, 'header')
    }
  }
})

test('newlines', (assert) => {
  assert.true(newlines.parseAll('\n\n\n').success)
  assert.false(newlines.parseAll('\n ').success)
})

test('paragraph', (assert) => {
  assert.true(paragraph.parseAll('test').success)

  {
    const result = paragraph.apply('te\ns*t*\n\n', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result.value[0].value, 'te\ns')
      assert.equal(result.result.value[1].value, 't')
    }
  }
})
