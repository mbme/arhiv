import {
  regex,
  satisfy,
  expect,
  bof,
  anyCharExcept,
  everythingUntil,
  ParserResult,
} from '~/parser-combinator'
import {
  trimLeft,
} from '~/utils'

interface INode<T extends string> {
  type: T
}

interface INodeString extends INode<'String'> {
  value: string
}
function isCharNode(node: INode<any>): node is INodeChar {
  return node.type === 'Char'
}

const groupCharsIntoStrings = <T extends INode<string>>(nodes: Array<INodeChar | T>) => { // group chars into strings
  const values: Array<INodeString | T> = []

  let str = ''
  for (const node of nodes) {
    if (isCharNode(node)) {
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
}

const newline = expect('\n')

// FIXME handle escaped chars like \*

// some *bold* text
interface INodeBold extends INode<'Bold'> {
  value: string
}
export const bold = anyCharExcept('*\n').oneOrMore().inside(expect('*'))
  .map((value): INodeBold => ({ type: 'Bold', value: value.join('') }), 'Bold')

// some `monospace` text
interface INodeMono extends INode<'Mono'> {
  value: string
}
export const mono = anyCharExcept('`\n').oneOrMore().inside(expect('`'))
  .map((value): INodeMono => ({ type: 'Mono', value: value.join('') }), 'Mono')

// some ~striketrough~ text
interface INodeStrikethrough extends INode<'Striketrough'> {
  value: string
}
export const strikethrough = anyCharExcept('~\n').oneOrMore().inside(expect('~'))
  .map((value): INodeStrikethrough => ({ type: 'Striketrough', value: value.join('') }), 'Strikethrough')

// [[link][with optional description]]
interface INodeLink extends INode<'Link'> {
  link: string
  description: string
}
const linkPart = anyCharExcept(']\n').oneOrMore().between(expect('['), expect(']'))
  .map(value => value.join(''))
export const link = linkPart.andThen(linkPart.optional()).between(expect('['), expect(']'))
  .map((value): INodeLink => ({
    type: 'Link',
    link: value[0],
    description: value[1] || '',
  }), 'Link')
type LinkType = ParserResult<typeof link>

const inlineElements = bold.orElse(mono).orElse(strikethrough).orElse(link)
type InlineElementsType = ParserResult<typeof inlineElements>

// # Header lvl 1 or ## Header lvl 2
interface INodeHeader extends INode<'Header'> {
  value: string
  level: 1 | 2
}
export const header = bof.orElse(newline).andThen(regex(/^#{1,2} .*/))
  .map((value): INodeHeader => {
    const headerStr = value[1]
    const level = headerStr.startsWith('## ') ? 2 : 1

    return {
      type: 'Header',
      value: trimLeft(headerStr, '# '),
      level,
    }
  }, 'Header')

interface INodeChar extends INode<'Char'> {
  value: string
}

// * unordered list
const listChar = satisfy((msg, pos) => {
  if (msg[pos] === '\n' && msg[pos + 1] === '\n') {
    return [false, 'found newlines']
  }

  if (msg[pos] === '\n' && msg[pos + 1] === '*') {
    return [false, 'found new list item']
  }

  return [true, msg[pos]]
}).map((value): INodeChar => ({ type: 'Char', value }), 'Char')

interface INodeListItem extends INode<'ListItem'> {
  children: Array<InlineElementsType | INodeString>
}
const unorderedListItem =
  inlineElements.orElse(listChar)
    .oneOrMore()
    .map(groupCharsIntoStrings)
    .map((value): INodeListItem => ({ type: 'ListItem', children: value }), 'ListItem')

interface INodeUnorderedList extends INode<'UnorderedList'> {
  listItems: INodeListItem[]
}
export const unorderedList = bof.orElse(newline)
  .andThen(expect('* '))
  .dropAndThen(unorderedListItem)
  .oneOrMore()
  .map((value): INodeUnorderedList => ({ type: 'UnorderedList', listItems: value }), 'UnorderedList')

// ```js
// codeBlock()
// ```
interface INodeCodeBlock extends INode<'CodeBlock'> {
  lang: string
  code: string
}
export const codeBlock = bof.orElse(newline).andThen(expect('```'))
  .dropAndThen(everythingUntil(newline)) // lang
  .andThen(everythingUntil(expect('\n```'))) // code
  .map((value): INodeCodeBlock => ({ type: 'CodeBlock', lang: value[0], code: value[1] }), 'CodeBlock')

const paragraphChar = satisfy((msg, pos) => {
  if (msg[pos] === '\n' && msg[pos + 1] === '\n') {
    return [false, 'found newlines']
  }

  return [true, msg[pos]]
}).map((value): INodeChar => ({ type: 'Char', value }), 'Char')

interface INodeParagraph extends INode<'Paragraph'> {
  children: Array<INodeHeader | INodeUnorderedList | INodeCodeBlock | InlineElementsType | INodeString>
}
export const paragraph = header
  .orElse(unorderedList)
  .orElse(codeBlock)
  .orElse(inlineElements)
  .orElse(paragraphChar)
  .oneOrMore()
  .map(groupCharsIntoStrings)
  .map((value): INodeParagraph => ({ type: 'Paragraph', children: value }), 'Paragraph')

interface INodeNewlines extends INode<'Newlines'> { }
export const newlines = regex(/^\n{2,}/)
  .map((): INodeNewlines => ({ type: 'Newlines' }), 'Newlines')

interface INodeMarkup extends INode<'Markup'> {
  children: Array<INodeNewlines | INodeParagraph>
}
export const markupParser =
  newlines
    .orElse(paragraph)
    .zeroOrMore()
    .map((value): INodeMarkup => ({ type: 'Markup', children: value }), 'Markup')

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


export const selectLinks = (node: INode<any>): LinkType[] => select('Link', node)
