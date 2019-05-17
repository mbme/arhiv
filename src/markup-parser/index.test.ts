import { test } from '~/tester'
import {
  newlines,
  paragraph,
  mono,
  bold,
  strikethrough,
  header,
  link,
  unorderedList,
  codeBlock,
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

  {
    const result = bold.apply('*test**', 0)
    assert.true(result.success)
    if (result.success) {
      assert.equal(result.result.value, 'test')
    }
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

test('unordered list', (assert) => {
  {
    const result = unorderedList.parseAll('* test')
    assert.true(result.success)
    if (result.success) {
      const items = result.result.value

      assert.equal(items.length, 1)
      assert.equal(items[0].value[0].value, 'test')
    }
  }

  {
    const result = unorderedList.parseAll('* test\ntest\n* ok *go*')
    assert.true(result.success)
    if (result.success) {
      const items = result.result.value
      assert.equal(items.length, 2)
      assert.equal(items[0].value[0].value, 'test\ntest')

      assert.equal(items[1].value[0].value, 'ok ')
      assert.equal(items[1].value[1].type, 'Bold')
      assert.equal(items[1].value[1].value, 'go')
    }
  }
})

test('link', (assert) => {
  {
    const result = link.parseAll('[[url][description]]')
    assert.true(result.success)
    if (result.success) {
      const [url, description] = result.result.value
      assert.equal(url, 'url')
      assert.equal(description, 'description')
    }
  }

  {
    const result = link.parseAll('[[url]]')
    assert.true(result.success)
    if (result.success) {
      const [url, description] = result.result.value
      assert.equal(url, 'url')
      assert.equal(description, undefined)
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

test('code block', (assert) => {
  const result = codeBlock.apply('```js\ntest\n```', 0)
  assert.true(result.success)
  if (result.success) {
    const [lang, code] = result.result.value
    assert.equal(lang, 'js')
    assert.equal(code, 'test')
  }
})
