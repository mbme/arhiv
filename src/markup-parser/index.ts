import {
  regex,
  satisfy,
  expect,
  bof,
  anyCharExcept,
  everythingUntil,
  select,
  INode,
  ParserResult,
} from '~/parser-combinator'
import { trimLeft } from '~/utils'
import { groupCharsIntoStrings } from './utils'

const newline = expect('\n')

// FIXME handle escaped chars like \*

// some *bold* text
export const bold = anyCharExcept('*\n').oneOrMore().between(expect('*'), expect('*'))
  .map(value => value.join(''))
  .asNode('Bold')

// some `monospace` text
export const mono = anyCharExcept('`\n').oneOrMore().between(expect('`'), expect('`'))
  .map(value => value.join(''))
  .asNode('Mono')

// some ~striketrough~ text
export const strikethrough = anyCharExcept('~\n').oneOrMore().between(expect('~'), expect('~'))
  .map(value => value.join(''))
  .asNode('Strikethrough')

// [[link][with optional description]]
const linkPart = anyCharExcept(']\n').oneOrMore().between(expect('['), expect(']')).map(value => value.join(''))
export const link = linkPart.andThen(linkPart.optional()).between(expect('['), expect(']'))
  .asNode('Link')
type LinkType = ParserResult<typeof link>

const inlineElements = bold.orElse(mono).orElse(strikethrough).orElse(link)

// # Header lvl 1 or ## Header lvl 2
export const header = bof.orElse(newline).andThen(regex(/^#{1,2} .*/))
  .map(value => {
    const headerStr = value[1]
    const level = headerStr.startsWith('## ') ? 2 : 1

    return [level, trimLeft(headerStr, '# ')] as [number, string]
  })
  .asNode('Header')

// * unordered list
const listChar = satisfy((msg, pos) => {
  if (msg[pos] === '\n' && msg[pos + 1] === '\n') {
    return [false, 'found newlines']
  }

  if (msg[pos] === '\n' && msg[pos + 1] === '*') {
    return [false, 'found new list item']
  }

  return [true, msg[pos]]
}).asNode('Char')

const unorderedListItem =
  inlineElements.orElse(listChar)
    .oneOrMore()
    .map(groupCharsIntoStrings)

export const unorderedList = bof.orElse(newline)
  .andThen(expect('* '))
  .andThen(unorderedListItem)
  .map(value => value[1]).asNode('ListItem')
  .oneOrMore()
  .asNode('UnorderedList')

// ```js
// codeBlock()
// ```
export const codeBlock = bof.orElse(newline).andThen(expect('```'))
  .andThen(everythingUntil(newline)).map(value => value[1]) // lang
  .andThen(everythingUntil(expect('\n```')))
  .asNode('CodeBlock')

const paragraphChar = satisfy((msg, pos) => {
  if (msg[pos] === '\n' && msg[pos + 1] === '\n') {
    return [false, 'found newlines']
  }

  return [true, msg[pos]]
}).asNode('Char')

export const paragraph = header
  .orElse(unorderedList)
  .orElse(codeBlock)
  .orElse(inlineElements)
  .orElse(paragraphChar)
  .oneOrMore()
  .map(groupCharsIntoStrings)
  .asNode('Paragraph')

export const newlines = regex(/^\n{2,}/)
  .asNode('Newlines')

export const markupParser =
  newlines
    .orElse(paragraph)
    .zeroOrMore()
    .asNode('Markup')

export const selectLinks = (node: INode<any>): LinkType[] => select('Link', node)
