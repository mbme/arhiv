import {
  regex,
  satisfy,
} from '~/parser-combinator'
import { trim } from '~/utils'

export const newlines = regex(/^\n{2,}/)
  .asNode('Newlines')

export const bold = regex(/^\*.*\*/)
  .map(value => trim(value, '*'))
  .asNode('Bold')

export const mono = regex(/^`.*`/)
  .map(value => trim(value, '`'))
  .asNode('Mono')

const paragraphChar = satisfy((msg, pos) => {
  if (msg[pos] === '\n' && msg[pos + 1] === '\n') {
    return [false, 'found newlines']
  }

  return [true, msg[pos]]
}).asNode('ParagraphChar')

export const paragraph = bold.orElse(mono).orElse(paragraphChar).oneOrMore()
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
