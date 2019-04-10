import { test } from '~/tester'
import { parse } from '~/parser-combinator'
import {
  newlines,
  paragraph,
} from './index'

test('newlines', (assert) => {
  assert.true(parse(newlines, '\n\n\n').success)
  assert.false(parse(newlines, '\n ').success)
})

test('paragraph', (assert) => {
  assert.true(paragraph('test', 0).success)
  {
    const result = paragraph('test\n\n', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result, 'test')
    }
  }
})
