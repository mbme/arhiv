import {
  regex,
  satisfy,
  expect,
  bof,
  anyCharExcept,
} from '~/parser-combinator'
import { trimLeft } from '~/utils'

const newline = expect('\n')

export const newlines = regex(/^\n{2,}/)
  .asNode('Newlines')

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

// # Header lvl 1 or ## Header lvl 2
export const header = bof.orElse(newline).andThen(regex(/^#{1,2} .*/))
  .map(value => {
    const headerStr = value[1]
    const level = headerStr.startsWith('## ') ? 2 : 1

    return [level, trimLeft(headerStr, '# ')]
  })
  .asNode('Header')

const linkPart = anyCharExcept(']\n').oneOrMore().between(expect('['), expect(']')).map(value => value.join(''))
export const link = linkPart.andThen(linkPart.optional()).between(expect('['), expect(']'))
  .asNode('Link')

const paragraphChar = satisfy((msg, pos) => {
  if (msg[pos] === '\n' && msg[pos + 1] === '\n') {
    return [false, 'found newlines']
  }

  return [true, msg[pos]]
}).asNode('ParagraphChar')

export const paragraph = header
  .orElse(bold)
  .orElse(mono)
  .orElse(strikethrough)
  .orElse(link)
  .orElse(paragraphChar)
  .oneOrMore()
  .map(nodes => { // group chars into strings
    const values = []
    let str = ''
    for (const node of nodes) {
      if (node.type === 'ParagraphChar') {
        str += node.value
        continue
      }

      if (str.length) {
        values.push({ type: 'String', value: str })
        str = ''
      }

      values.push(node)
    }

    if (str.length) {
      values.push({ type: 'String', value: str })
    }

    return values
  })
  .asNode('Paragraph')

export const markupParser = newlines.orElse(paragraph).zeroOrMore()
