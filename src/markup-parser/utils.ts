import {
  INode,
  inode,
} from '~/parser-combinator'

export const groupCharsIntoStrings = (nodes: Array<INode<any>>) => { // group chars into strings
  const values = []
  let str = ''
  for (const node of nodes) {
    if (node.type === 'Char') {
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
