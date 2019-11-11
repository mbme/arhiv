import {
  test,
  assert,
  assertInstanceOf,
} from '~/tester'
import {
  assertSuccess,
  assertFailure,
} from '~/parser-combinator/test-utils'
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
} from './parsers'
import {
  NodeString,
  NodeBold,
} from './nodes'

test('inline', () => {
  {
    const result = mono.parseAll('`test`')
    assertSuccess(result)
    assert.equal(result.value.value, 'test')
  }

  assertFailure(mono.parseAll('`te\nst`'))

  {
    const result = bold.apply('*test**', 0)
    assertSuccess(result)
    assert.equal(result.value.value, 'test')
  }

  assertSuccess(bold.parseAll('*test*'))
  assertSuccess(strikethrough.parseAll('~test~'))
})

test('header', () => {
  {
    const result = header.parseAll('# header')
    assertSuccess(result)
    assert.equal(result.value.level, 1)
    assert.equal(result.value.value, 'header')
  }

  {
    const result = header.apply('test\n## header\ntest', 4)
    assertSuccess(result)
    assert.equal(result.value.level, 2)
    assert.equal(result.value.value, 'header')
  }
})

test('unordered list', () => {
  {
    const result = unorderedList.parseAll('* test')
    assertSuccess(result)

    assert.equal(result.value.children.length, 1)

    const item = result.value.children[0].children[0]
    assertInstanceOf(item, NodeString)
    assert.equal(item.value, 'test')
  }

  {
    const result = unorderedList.parseAll('* test\ntest\n* ok *go*')
    assertSuccess(result)
    assert.equal(result.value.children.length, 2)

    {
      const item = result.value.children[0].children[0]
      assertInstanceOf(item, NodeString)
      assert.equal(item.value, 'test\ntest')
    }

    {
      const item = result.value.children[1].children[0]
      assertInstanceOf(item, NodeString)
      assert.equal(item.value, 'ok ')
    }

    {
      const item = result.value.children[1].children[1]
      assertInstanceOf(item, NodeBold)
      assert.equal(item.value, 'go')
    }
  }
})

test('link', () => {
  {
    const result = link.parseAll('[[url][description]]')
    assertSuccess(result)
    assert.equal(result.value.link, 'url')
    assert.equal(result.value.description, 'description')
  }

  {
    const result = link.parseAll('[[url]]')
    assertSuccess(result)
    assert.equal(result.value.link, 'url')
    assert.equal(result.value.description, '')
  }
})

test('newlines', () => {
  assertSuccess(newlines.parseAll('\n\n\n'))
  assertFailure(newlines.parseAll('\n '))
})

test('paragraph', () => {
  assertSuccess(paragraph.parseAll('test'))

  {
    const result = paragraph.apply('te\ns*t*\n\n', 0)
    assertSuccess(result)

    {
      const item = result.value.children[0]
      assertInstanceOf(item, NodeString)
      assert.equal(item.value, 'te\ns')
    }

    {
      const item = result.value.children[1]
      assertInstanceOf(item, NodeBold)
      assert.equal(item.value, 't')
    }
  }
})

test('code block', () => {
  const result = codeBlock.apply('```js\ntest\n```', 0)
  assertSuccess(result)
  assert.equal(result.value.lang, 'js')
  assert.equal(result.value.value, 'test')
})
