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
  assertSuccess(mono.parseAll('`test`'), (result) => {
    assert.equal(result.value, 'test')
  })

  assertFailure(mono.parseAll('`te\nst`'))

  assertSuccess(bold.apply('*test**', 0), (result) => {
    assert.equal(result.value, 'test')
  })

  assertSuccess(bold.parseAll('*test*'))
  assertSuccess(strikethrough.parseAll('~test~'))
})

test('header', () => {
  assertSuccess(header.parseAll('# header'), ({ level, value }) => {
    assert.equal(level, 1)
    assert.equal(value, 'header')
  })

  assertSuccess(header.apply('test\n## header\ntest', 4), ({ level, value }) => {
    assert.equal(level, 2)
    assert.equal(value, 'header')
  })
})

test('unordered list', () => {
  assertSuccess(unorderedList.parseAll('* test'), (result) => {
    assert.equal(result.children.length, 1)

    const item = result.children[0].children[0]
    assert.true(item instanceof NodeString)
    if (item instanceof NodeString) {
      assert.equal(item.value, 'test')
    }
  })

  assertSuccess(unorderedList.parseAll('* test\ntest\n* ok *go*'), (result) => {
    assert.equal(result.children.length, 2)

    {
      const item = result.children[0].children[0]
      assertInstanceOf(item, NodeString)
      assert.equal(item.value, 'test\ntest')
    }

    {
      const item = result.children[1].children[0]
      assert.true(item instanceof NodeString)
      if (item instanceof NodeString) {
        assert.equal(item.value, 'ok ')
      }
    }

    {
      const item = result.children[1].children[1]
      assert.true(item instanceof NodeBold)
      if (item instanceof NodeBold) {
        assert.equal(item.value, 'go')
      }
    }
  })
})

test('link', () => {
  assertSuccess(link.parseAll('[[url][description]]'), (result) => {
    assert.equal(result.link, 'url')
    assert.equal(result.description, 'description')
  })

  assertSuccess(link.parseAll('[[url]]'), (result) => {
    assert.equal(result.link, 'url')
    assert.equal(result.description, '')
  })
})

test('newlines', () => {
  assertSuccess(newlines.parseAll('\n\n\n'))
  assertFailure(newlines.parseAll('\n '))
})

test('paragraph', () => {
  assertSuccess(paragraph.parseAll('test'))

  assertSuccess(paragraph.apply('te\ns*t*\n\n', 0), (result) => {
    {
      const item = result.children[0]
      assert.true(item instanceof NodeString)
      if (item instanceof NodeString) {
        assert.equal(item.value, 'te\ns')
      }
    }

    {
      const item = result.children[1]
      assert.true(item instanceof NodeBold)
      if (item instanceof NodeBold) {
        assert.equal(item.value, 't')
      }
    }
  })
})

test('code block', () => {
  assertSuccess(codeBlock.apply('```js\ntest\n```', 0), ({ lang, value }) => {
    assert.equal(lang, 'js')
    assert.equal(value, 'test')
  })
})
