import {
  regex,
  satisfy,
  expect,
  bof,
  anyCharExcept,
  everythingUntil,
} from '~/parser-combinator'
import {
  trimLeft,
} from '~/utils'

abstract class Node {
  getChildren<N extends Node>(): N[] {
    return []
  }
}

class NodeChar extends Node {
  constructor(public value: string) {
    super()
  }
}

class NodeString extends Node {
  constructor(public value: string) {
    super()
  }
}

class NodeBold extends Node {
  constructor(public value: string) {
    super()
  }
}

class NodeMono extends Node {
  constructor(public value: string) {
    super()
  }
}

class NodeStrikethrough extends Node {
  constructor(public value: string) {
    super()
  }
}

class NodeLink extends Node {
  constructor(
    public link: string,
    public description: string,
  ) {
    super()
  }
}

type InlineNodes = NodeBold | NodeMono | NodeStrikethrough | NodeLink | NodeString

class NodeListItem extends Node {
  constructor(public children: InlineNodes[]) {
    super()
  }
}

class NodeUnorderedList extends Node {
  constructor(public children: NodeListItem[]) {
    super()
  }
}

class NodeHeader extends Node {
  constructor(
    public value: string,
    public level: 1 | 2,
  ) {
    super()
  }
}

class NodeCodeBlock extends Node {
  constructor(
    public lang: string,
    public value: string,
  ) {
    super()
  }
}

class NodeParagraph extends Node {
  constructor(public children: Array<NodeHeader | NodeUnorderedList | NodeCodeBlock | InlineNodes>) {
    super()
  }
}

class NodeNewlines extends Node { }

class NodeMarkup extends Node {
  constructor(public children: Array<NodeNewlines | NodeParagraph>) {
    super()
  }
}

const groupCharsIntoStrings = <N extends Node>(nodes: Array<NodeChar | N>) => { // group chars into strings
  const values: Array<NodeString | N> = []

  let str = ''
  for (const node of nodes) {
    if (node instanceof NodeChar) {
      str += node.value
      continue
    }

    if (str.length) {
      values.push(new NodeString(str))
      str = ''
    }

    values.push(node)
  }

  if (str.length) {
    values.push(new NodeString(str))
  }

  return values
}

const newline = expect('\n')

// FIXME handle escaped chars like \*

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

const unorderedListItem =
  inlineElements.orElse(listChar)
    .oneOrMore()
    .map(groupCharsIntoStrings)
    .map(value => new NodeListItem(value), 'ListItem')

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

export const markupParser =
  newlines
    .orElse(paragraph)
    .zeroOrMore()
    .map(value => new NodeMarkup(value), 'Markup')

// TODO use classes for Nodes instead of interfaces, + method to iterate children
// TODO generator for the markupParser

// const select = (type: string, node: INode<any>): Array<INode<any>> => {
//   if (node.type === type) {
//     return [node]
//   }

//   if (isArray(node.value)) {
//     const children = node.value as Array<INode<any>>

//     return children.flatMap(value => select(type, value))
//   }

//   return []
// }

// export const selectLinks = (node: INode<any>): LinkType[] => select('Link', node)
