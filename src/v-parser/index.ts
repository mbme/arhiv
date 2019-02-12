import log from '../logger'
import { uniq, isSha256, isSubSequence, isString } from '../utils'

export enum NodeType {
  Bold = 'Bold',
  Mono = 'Mono',
  Strikethrough = 'Strikethrough',
  Header = 'Header',
  LinkPart = 'LinkPart',
  Link = 'Link',
  ListItem = 'ListItem',
  CodeBlock = 'CodeBlock',
  Paragraph = 'Paragraph',
  Document = 'Document',
}

type LinkType = 'url' | 'image'

export interface INodeBold {
  type: NodeType.Bold
  text: string
}
export interface INodeMono {
  type: NodeType.Mono
  text: string
}
export interface INodeStrikethrough {
  type: NodeType.Strikethrough
  text: string
}
interface INodeLinkPart {
  type: NodeType.LinkPart
  text: string
}
export interface INodeLink {
  type: NodeType.Link
  linkType: LinkType
  name: string
  address: string
  isInternal: boolean
}

type ListItemChild =
  INodeBold
  | INodeMono
  | INodeStrikethrough
  | INodeLink
  | string
export interface INodeListItem {
  type: NodeType.ListItem
  items: ListItemChild[]
}

export interface INodeCodeBlock {
  type: NodeType.CodeBlock
  source?: string
  lang?: string
  text: string
}
export interface INodeHeader {
  type: NodeType.Header
  text: string
  lvl: 1 | 2
}

type ParagraphChild =
  INodeHeader
  | INodeListItem
  | INodeCodeBlock
  | INodeBold
  | INodeMono
  | INodeStrikethrough
  | INodeLink
export interface INodeParagraph {
  type: NodeType.Paragraph
  items: ParagraphChild[]
}

export interface INodeDocument {
  type: NodeType.Document
  items: INodeParagraph[]
}

export type Node =
  string
  | INodeDocument
  | INodeParagraph
  | INodeHeader
  | INodeListItem
  | INodeCodeBlock
  | INodeBold
  | INodeMono
  | INodeStrikethrough
  | INodeLink
  | INodeLinkPart

interface IElementParser {
  children: NodeType[]
  isValid(children: Node[]): boolean
  isStart(str: string, pos: number): boolean
  isBreak(str: string, pos: number): boolean
  isEnd(str: string, pos: number): boolean
  skip: [number, number]
  escapeChar: string
  postprocess(children: Node[]): Node
}

// BOF === Beginning Of File
const isAfterNewlineOrBOF = (str: string, i: number) => (str[i - 1] === '\n' || i === 0)
const isCharSeq = (seq: string) => (str: string, i: number) => isSubSequence(str, i, seq)
const isNewline = isCharSeq('\n')
const NEVER = () => false
const isNotEmpty = (children: Node[]) => children.length > 0

function parseLink(s: string): { linkType: LinkType, address: string } {
  if (s.startsWith('image:')) {
    return {
      linkType: 'image',
      address: s.substring(6),
    }
  }

  return {
    linkType: 'url',
    address: s,
  }
}

const INLINE_TYPES = [NodeType.Bold, NodeType.Mono, NodeType.Strikethrough, NodeType.Link]

const Grammar: { [key in NodeType]: IElementParser } = {
  [NodeType.Bold]: { // some *bold* text
    children: [],
    skip: [1, 1],
    escapeChar: '*',
    isStart: isCharSeq('*'),
    isBreak: isNewline,
    isEnd: isCharSeq('*'),
    isValid: isNotEmpty,
    postprocess(children: Node[]): INodeBold {
      return {
        type: NodeType.Bold,
        text: children[0] as string,
      }
    },
  },

  [NodeType.Mono]: { // some `monospace` text
    children: [],
    skip: [1, 1],
    escapeChar: '`',
    isStart: isCharSeq('`'),
    isBreak: isNewline,
    isEnd: isCharSeq('`'),
    isValid: isNotEmpty,
    postprocess(children: Node[]): INodeMono {
      return {
        type: NodeType.Mono,
        text: children[0] as string,
      }
    },
  },

  [NodeType.Strikethrough]: { // some ~striketrough~ text
    children: [],
    skip: [1, 1],
    escapeChar: '~',
    isStart: isCharSeq('~'),
    isBreak: isNewline,
    isEnd: isCharSeq('~'),
    isValid: isNotEmpty,
    postprocess(children: Node[]): INodeStrikethrough {
      return {
        type: NodeType.Strikethrough,
        text: children[0] as string,
      }
    },
  },

  [NodeType.Header]: { // # Header lvl 1 or ## Header lvl 2
    skip: [0, 0],
    escapeChar: '',
    children: [],
    isBreak: NEVER,
    isStart: (str: string, pos: number) => isAfterNewlineOrBOF(str, pos)
      && (isSubSequence(str, pos, '# ') || isSubSequence(str, pos, '## ')),
    isEnd: (str: string, pos: number) => pos === str.length || str[pos] === '\n',
    isValid: isNotEmpty,
    postprocess(children: Node[]): INodeHeader {
      const text = children[0] as string
      const lvl = text.startsWith('# ') ? 1 : 2

      return {
        type: NodeType.Header,
        lvl,
        text: text.substring(lvl + 1),
      }
    },
  },

  [NodeType.LinkPart]: {
    children: [],
    isValid: () => true,
    skip: [1, 1],
    escapeChar: ']',
    isStart: isCharSeq('['),
    isBreak: isNewline,
    isEnd: isCharSeq(']'),
    postprocess(children: Node[]): INodeLinkPart {
      return {
        type: NodeType.LinkPart,
        text: children[0] as string,
      }
    },
  },

  [NodeType.Link]: { // links [[type:ref][name]] or [[type:ref]]
    isBreak: NEVER,
    skip: [1, 1],
    escapeChar: '',
    children: [NodeType.LinkPart],
    isStart: isCharSeq('['),
    isEnd: isCharSeq(']'),
    isValid(children: Node[]) {
      if (children.length !== 1 && children.length !== 2) return false

      // ensure link has only LinkParts
      return children.filter(item => isString(item) || item.type !== NodeType.LinkPart).length === 0
    },
    postprocess(children: Node[]): INodeLink {
      const [addressItem, nameItem] = children as INodeLinkPart[]

      const {
        linkType,
        address,
      } = parseLink(addressItem.text)

      return {
        type: NodeType.Link,
        linkType,
        name: nameItem ? nameItem.text : '',
        address,
        isInternal: isSha256(address),
      }
    },
  },

  [NodeType.ListItem]: { // * Unordered list
    isBreak: NEVER,
    isValid: () => true,
    escapeChar: '',
    children: [...INLINE_TYPES],
    skip: [1, 0],
    isStart: (str: string, pos: number) => isAfterNewlineOrBOF(str, pos) && isSubSequence(str, pos, '* '),
    isEnd: (str: string, pos: number) => pos === str.length || str[pos] === '\n',
    postprocess(children: Node[]): INodeListItem {
      return {
        type: NodeType.ListItem,
        items: children as ListItemChild[],
      }
    },
  },

  // ```js
  // callFunc();
  // ```
  [NodeType.CodeBlock]: {
    children: [],
    isBreak: NEVER,
    isValid: () => true,
    escapeChar: '',
    skip: [3, 3],
    isStart: (str: string, pos: number) => isAfterNewlineOrBOF(str, pos) && isSubSequence(str, pos, '```'),
    isEnd: (str: string, pos: number) => isSubSequence(str, pos, '\n```'),
    postprocess(children: Node[]): INodeCodeBlock {
      const text = children[0] as string
      const firstNewlinePos = text.indexOf('\n')
      const lang = text.substring(0, firstNewlinePos)

      if (lang.startsWith('quote:')) { // handle quotes
        return {
          type: NodeType.CodeBlock,
          source: lang.substring(6),
          text: text.substring(firstNewlinePos + 1),
        }
      }

      return {
        type: NodeType.CodeBlock,
        lang,
        text: text.substring(firstNewlinePos + 1),
      }
    },
  },

  [NodeType.Paragraph]: {
    skip: [0, 0],
    isBreak: NEVER,
    isValid: () => true,
    escapeChar: '',
    children: [NodeType.Header, NodeType.ListItem, NodeType.CodeBlock, ...INLINE_TYPES],
    isStart: (str: string, pos: number) => pos === 0 || (str[pos] !== '\n' && str[pos - 1] === '\n'),
    isEnd(str: string, pos: number) {
      if (pos === str.length) return true

      const ending = str.slice(pos, pos + 2)
      if (ending === '\n' || ending === '\n\n') return true

      return false
    },
    postprocess(children: Node[]): INodeParagraph {
      return {
        type: NodeType.Paragraph,
        items: children as ParagraphChild[],
      }
    },
  },

  [NodeType.Document]: {
    skip: [0, 0],
    escapeChar: '',
    isBreak: NEVER,
    isValid: () => true,
    children: [NodeType.Paragraph],
    isStart: (_: string, pos: number) => pos === 0,
    isEnd: (str: string, pos: number) => pos === str.length,
    postprocess(children: Node[]): INodeDocument {
      return {
        type: NodeType.Document,
        items: children as INodeParagraph[],
      }
    },
  },
}

export function parseFrom(str: string, pos: number, type: NodeType): [number, Node?] {
  const rule = Grammar[type]
  const [skipStart, skipEnd] = rule.skip

  let i = pos
  if (!rule.isStart(str, i)) return [0, undefined]

  i += skipStart

  const children: Node[] = []
  let text = ''
  let ended = false

  outer:
  while (true) {
    // handle escapes
    if (str[i] === '\\' && rule.escapeChar === str[i + 1]) {
      text += str[i + 1]
      i += 2
      continue outer
    }

    if (str[i] === '\r') { // ignore \r
      i += 1
      continue outer
    }

    if (rule.isEnd(str, i)) {
      ended = true
      i += skipEnd
      break outer
    }

    if (i === str.length) break outer

    inner:
    for (const childType of rule.children) {
      const [length, leaf] = parseFrom(str, i, childType)

      if (!length) continue inner

      if (text) {
        children.push(text)
        text = ''
      }

      i += length
      children.push(leaf!)

      continue outer
    }

    if (rule.isBreak(str, i)) return [0, undefined]

    text += str[i]
    i += 1
  }

  if (text) children.push(text)

  // validate result
  if (!ended || !rule.isValid(children)) return [0, undefined]

  return [i - pos, rule.postprocess(children)]
}

export function parse(str: string) {
  const [i, tree] = parseFrom(str, 0, NodeType.Document)

  if (i !== str.length) {
    log.warn(`parser covers ${i} out of ${str.length} chars`)
  }

  return tree as INodeDocument
}

export function select(tree: Node, type: NodeType): Node[] {
  const result: Node[] = []

  if (!tree || isString(tree)) return result

  if (tree.type === type) result.push(tree)

  const items = (tree as any).items as Node[] || []
  items.forEach((child: Node) => result.push(...select(child, type)))

  return result
}

export function extractFileIds(tree: Node) {
  const links = select(tree, NodeType.Link) as INodeLink[]
  const ids = links.reduce<string[]>((acc, link) => {
    if (link.isInternal) acc.push(link.address)

    return acc
  }, [])

  return uniq(ids)
}

export const createLink = (name = '', link: string) => name ? `[[${link}][${name}]]` : `[[${link}]]`
export const createImageLink = (name: string, link: string) => createLink(name, `image:${link}`)
