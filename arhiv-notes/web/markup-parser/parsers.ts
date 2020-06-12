import {
  regex,
  satisfy,
  expect,
  bof,
  anyCharExcept,
  everythingUntil,
} from '@v/parser-combinator'
import {
  trimLeft,
} from '@v/utils'
import {
  NodeBold,
  NodeMono,
  NodeStrikethrough,
  NodeChar,
  NodeLink,
  NodeHeader,
  NodeListItem,
  NodeUnorderedList,
  NodeCodeBlock,
  NodeParagraph,
  NodeNewlines,
  NodeMarkup,
} from './nodes'

import { groupCharsIntoStrings } from './utils'

// TODO handle escaped chars like \*

const newline = expect('\n')

// some *bold* text
export const bold = anyCharExcept('*\n').oneOrMore().inside(expect('*'))
  .map(value => new NodeBold(value.join('')), 'Bold')

// some `monospace` text
export const mono = anyCharExcept('`\n').oneOrMore().inside(expect('`'))
  .map(value => new NodeMono(value.join('')), 'Mono')

// some ~striketrough~ text
export const strikethrough = anyCharExcept('~\n').oneOrMore().inside(expect('~'))
  .map(value => new NodeStrikethrough(value.join('')), 'Strikethrough')

// [[link][with optional description]]
const linkPart = anyCharExcept(']\n').oneOrMore().between(expect('['), expect(']'))
  .map(value => value.join(''))

export const link = linkPart.andThen(linkPart.optional()).between(expect('['), expect(']'))
  .map(value => new NodeLink(value[0], value[1] || ''), 'Link')

const inlineElements = bold.orElse(mono).orElse(strikethrough).orElse(link)

// # Header lvl 1 or ## Header lvl 2
export const header = bof.orElse(newline).dropAndThen(regex(/^#{1,2} .*/))
  .map((value) => {
    const level = value.startsWith('## ') ? 2 : 1

    return new NodeHeader(trimLeft(value, '# '), level)
  }, 'Header')

// * unordered list
const listChar = satisfy((msg, pos) => {
  if (msg[pos] === '\n' && msg[pos + 1] === '\n') {
    return [false, 'found newlines']
  }

  if (msg[pos] === '\n' && msg[pos + 1] === '*') {
    return [false, 'found new list item']
  }

  return [true, msg[pos]]
})
  .map(value => new NodeChar(value), 'Char')

const unorderedListItem = (
  inlineElements.orElse(listChar)
    .oneOrMore()
    .map(groupCharsIntoStrings)
    .map(value => new NodeListItem(value), 'ListItem')
)

export const unorderedList = bof.orElse(newline)
  .andThen(expect('* '))
  .dropAndThen(unorderedListItem)
  .oneOrMore()
  .map(value => new NodeUnorderedList(value), 'UnorderedList')

// ```js
// codeBlock()
// ```
export const codeBlock = bof.orElse(newline).andThen(expect('```'))
  .dropAndThen(everythingUntil(newline)) // lang
  .andThen(everythingUntil(expect('\n```'))) // code
  .map(value => new NodeCodeBlock(value[0], value[1]), 'CodeBlock')

const paragraphChar = satisfy((msg, pos) => {
  if (msg[pos] === '\n' && msg[pos + 1] === '\n') {
    return [false, 'found newlines']
  }

  return [true, msg[pos]]
})
  .map(value => new NodeChar(value), 'Char')

export const paragraph = header
  .orElse(unorderedList)
  .orElse(codeBlock)
  .orElse(inlineElements)
  .orElse(paragraphChar)
  .oneOrMore()
  .map(groupCharsIntoStrings)
  .map(value => new NodeParagraph(value), 'Paragraph')

export const newlines = regex(/^\n{2,}/)
  .map(() => new NodeNewlines(), 'Newlines')

export const markupParser = (
  newlines
    .orElse(paragraph)
    .zeroOrMore()
    .map(value => new NodeMarkup(value), 'Markup')
)
