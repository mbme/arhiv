import {
  INode,
  inode,
  isINode,
} from '~/parser-combinator'

export const groupCharsIntoStrings = <T, K>(nodes: Array<K | INode<T> | INode<string>>) => { // group chars into strings
  const values = []
  let str = ''
  for (const node of nodes) {
    if (isINode(node) && node.type === 'Char') {
      str += node.value
      continue
    }

    if (str.length) {
      values.push(inode('String', str))
      str = ''
    }

    values.push(node)
  }

  if (str.length) {
    values.push(inode('String', str))
  }

  return values
}

export const createLink = (url: string, text: string = '') => text ? `[[${url}][${text}]]` : `[[${url}]]`
