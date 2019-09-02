import {
  test,
  asserts,
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
    asserts.equal(result.value, 'test')
  })

  assertFailure(mono.parseAll('`te\nst`'))

  assertSuccess(bold.apply('*test**', 0), (result) => {
    asserts.equal(result.value, 'test')
  })

  assertSuccess(bold.parseAll('*test*'))
  assertSuccess(strikethrough.parseAll('~test~'))
})

test('header', () => {
  assertSuccess(header.parseAll('# header'), ({ level, value }) => {
    asserts.equal(level, 1)
    asserts.equal(value, 'header')
  })

  assertSuccess(header.apply('test\n## header\ntest', 4), ({ level, value }) => {
    asserts.equal(level, 2)
    asserts.equal(value, 'header')
  })
})

test('unordered list', () => {
  assertSuccess(unorderedList.parseAll('* test'), (result) => {
    asserts.equal(result.children.length, 1)

    const item = result.children[0].children[0]
    asserts.true(item instanceof NodeString)
    if (item instanceof NodeString) {
      asserts.equal(item.value, 'test')
    }
  })

  assertSuccess(unorderedList.parseAll('* test\ntest\n* ok *go*'), (result) => {
    asserts.equal(result.children.length, 2)

    {
      const item = result.children[0].children[0]
      asserts.true(item instanceof NodeString)
      if (item instanceof NodeString) {
        asserts.equal(item.value, 'test\ntest')
      }
    }

    {
      const item = result.children[1].children[0]
      asserts.true(item instanceof NodeString)
      if (item instanceof NodeString) {
        asserts.equal(item.value, 'ok ')
      }
    }

    {
      const item = result.children[1].children[1]
      asserts.true(item instanceof NodeBold)
      if (item instanceof NodeBold) {
        asserts.equal(item.value, 'go')
      }
    }
  })
})

test('link', () => {
  assertSuccess(link.parseAll('[[url][description]]'), (result) => {
    asserts.equal(result.link, 'url')
    asserts.equal(result.description, 'description')
  })

  assertSuccess(link.parseAll('[[url]]'), (result) => {
    asserts.equal(result.link, 'url')
    asserts.equal(result.description, '')
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
      asserts.true(item instanceof NodeString)
      if (item instanceof NodeString) {
        asserts.equal(item.value, 'te\ns')
      }
    }

    {
      const item = result.children[1]
      asserts.true(item instanceof NodeBold)
      if (item instanceof NodeBold) {
        asserts.equal(item.value, 't')
      }
    }
  })
})

test('code block', () => {
  assertSuccess(codeBlock.apply('```js\ntest\n```', 0), ({ lang, value }) => {
    asserts.equal(lang, 'js')
    asserts.equal(value, 'test')
  })
})
