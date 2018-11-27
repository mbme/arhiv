import log from '../logger';
import { uniq, isSha256, isSubSequence } from '../utils';

enum NodeType { Text = 'Text', Bold = 'Bold', Mono = 'Mono', Strikethrough = 'Strikethrough', Header = 'Header', LinkPart = 'LinkPart', Link = 'Link', ListItem = 'ListItem', CodeBlock = 'CodeBlock', Paragraph = 'Paragraph', Document = 'Document' }

type LinkType = 'url' | 'image'

interface INodeText {
  type: NodeType.Text
  text: string
}
interface INodeBold {
  type: NodeType.Bold
  text: string
}
interface INodeMono {
  type: NodeType.Mono
  text: string
}
interface INodeStrikethrough {
  type: NodeType.Strikethrough
  text: string
}
interface INodeLinkPart {
  type: NodeType.LinkPart
  text: string
}
interface INodeLink {
  type: NodeType.Link
  linkType: LinkType
  name: string
  address: string
  isInternal: boolean
}

type ListItemChildren = INodeBold | INodeMono | INodeStrikethrough | INodeLink | INodeText
interface INodeListItem {
  type: NodeType.ListItem
  items: ListItemChildren[]
}

interface INodeCodeBlock {
  type: NodeType.CodeBlock
  source?: string
  lang?: string
  text: string
}
interface INodeHeader {
  type: NodeType.Header
  text: string
  lvl: 1 | 2
}

type ParagraphChildren = INodeHeader | INodeListItem | INodeCodeBlock | INodeBold | INodeMono | INodeStrikethrough | INodeLink
interface INodeParagraph {
  type: NodeType.Paragraph
  items: ParagraphChildren[]
}

interface INodeDocument {
  type: NodeType.Document
  items: INodeParagraph[]
}

type VElement = INodeDocument | INodeParagraph | INodeHeader | INodeListItem | INodeCodeBlock | INodeBold | INodeMono | INodeStrikethrough | INodeLink | INodeLinkPart | INodeText

type ElementParser = {
  children: NodeType[]
  isValid: (children: VElement[]) => boolean
  isStart: (str: string, pos: number) => boolean
  isBreak: (str: string, pos: number) => boolean
  isEnd: (str: string, pos: number) => boolean
  skip: [number, number]
  escapeChar: string
  postprocess: (children: VElement[]) => VElement
}

// BOF === Beginning Of File
const isAfterNewlineOrBOF = (str: string, i: number) => (str[i - 1] === '\n' || i === 0);
const isCharSeq = (seq: string) => (str: string, i: number) => isSubSequence(str, i, seq);
const isNewline = isCharSeq('\n');
const NEVER = () => false
const isNotEmpty = (children: VElement[]) => children.length > 0

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

const INLINE_TYPES = [NodeType.Bold, NodeType.Mono, NodeType.Strikethrough, NodeType.Link];

const Grammar: { [key in NodeType]: ElementParser } = {
  [NodeType.Bold]: { // some *bold* text
    children: [],
    skip: [1, 1],
    escapeChar: '*',
    isStart: isCharSeq('*'),
    isBreak: isNewline,
    isEnd: isCharSeq('*'),
    isValid: isNotEmpty,
    postprocess(children: VElement[]): INodeBold {
      return {
        type: NodeType.Bold,
        text: (<INodeText>children[0]).text,
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
    postprocess(children: VElement[]): INodeMono {
      return {
        type: NodeType.Mono,
        text: (<INodeText>children[0]).text,
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
    postprocess(children: VElement[]): INodeStrikethrough {
      return {
        type: NodeType.Strikethrough,
        text: (<INodeText>children[0]).text,
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
    postprocess(children: VElement[]): INodeHeader {
      const { text } = <INodeText>children[0];
      const lvl = text.startsWith('# ') ? 1 : 2;
      return {
        type: NodeType.Header,
        lvl,
        text: text.substring(lvl + 1),
      };
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
    postprocess(children: VElement[]): INodeLinkPart {
      return {
        type: NodeType.LinkPart,
        text: (<INodeText>children[0]).text,
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
    isValid(children: VElement[]) {
      if (children.length !== 1 && children.length !== 2) return false;

      return children.filter(item => item.type !== NodeType.LinkPart).length === 0;
    },
    postprocess(children: INodeLinkPart[]): INodeLink {
      const [addressItem, nameItem] = children;

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
      };
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
    postprocess(children: ListItemChildren[]): INodeListItem {
      return {
        type: NodeType.ListItem,
        items: children,
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
    postprocess(children: VElement[]): INodeCodeBlock {
      const { text } = <INodeText>children[0]
      const firstNewlinePos = text.indexOf('\n');
      const lang = text.substring(0, firstNewlinePos);

      if (lang.startsWith('quote:')) { // handle quotes
        return {
          type: NodeType.CodeBlock,
          source: lang.substring(6),
          text: text.substring(firstNewlinePos + 1),
        };
      }

      return {
        type: NodeType.CodeBlock,
        lang,
        text: text.substring(firstNewlinePos + 1),
      };
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
      if (pos === str.length) return true;

      const ending = str.slice(pos, pos + 2);
      if (ending === '\n' || ending === '\n\n') return true;

      return false;
    },
    postprocess(children: ParagraphChildren[]): INodeParagraph {
      return {
        type: NodeType.Paragraph,
        items: children,
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
    postprocess(children: INodeParagraph[]): INodeDocument {
      return {
        type: NodeType.Document,
        items: children,
      }
    },
  },
};

export function parseFrom(str: string, pos: number, type: NodeType): [number, VElement?] {
  const rule = Grammar[type];
  const [skipStart, skipEnd] = rule.skip;

  let i = pos;
  if (!rule.isStart(str, i)) return [0, undefined];

  i += skipStart;

  const children: VElement[] = []
  let text = '';
  let ended = false;

  outer:
  while (true) {
    // handle escapes
    if (str[i] === '\\' && rule.escapeChar === str[i + 1]) {
      text += str[i + 1];
      i += 2;
      continue outer;
    }

    if (str[i] === '\r') { // ignore \r
      i += 1;
      continue outer;
    }

    if (rule.isEnd(str, i)) {
      ended = true;
      i += skipEnd;
      break outer;
    }

    if (i === str.length) break outer;

    inner:
    for (const childType of rule.children) {
      const [length, leaf] = parseFrom(str, i, childType);

      if (!length) continue inner;

      if (text) {
        children.push({ type: NodeType.Text, text });
        text = '';
      }

      i += length;
      children.push(leaf!)

      continue outer;
    }

    if (rule.isBreak(str, i)) return [0, undefined];

    text += str[i];
    i += 1;
  }

  if (text) children.push({ type: NodeType.Text, text });

  // validate result
  if (!ended || !rule.isValid(children)) return [0, undefined];

  const length = i - pos;

  return [length, rule.postprocess(children)];
}

export function parse(str: string) {
  const [i, tree] = parseFrom(str, 0, NodeType.Document);

  if (global.__DEVELOPMENT__ && i !== str.length) {
    log.warn(`parser covers ${i} out of ${str.length} chars`);
  }

  return tree;
}

export function select(tree: VElement, type: NodeType): VElement[] {
  const result: VElement[] = [];
  if (tree.type === type) result.push(tree);

  const items = (<any>tree).items || [];
  items.forEach((child: VElement) => result.push(...select(child, type)));

  return result;
}

export function extractFileIds(tree: VElement) {
  const ids = select(tree, NodeType.Link).reduce((acc, link: INodeLink) => {
    if (link.isInternal) acc.push(link.address);

    return acc;
  }, <string[]>[])

  return uniq<string>(ids);
}

export const createLink = (name = '', link: string) => name ? `[[${link}][${name}]]` : `[[${link}]]`;
export const createImageLink = (name: string, link: string) => createLink(name, `image:${link}`);
