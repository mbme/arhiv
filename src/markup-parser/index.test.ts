import { test } from '~/tester'
import {
  newlines,
  paragraph,
  mono,
  bold,
} from './index'

test('inline', (assert) => {
  assert.true(mono.parseAll('`test`').success)
  assert.false(mono.parseAll('`te\nst`').success)

  assert.true(bold.parseAll('*test*').success)
  assert.false(bold.parseAll('*te\nst*').success)
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
