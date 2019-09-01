import {
  Node,
  NodeString,
  NodeChar,
  NodeLink,
} from './nodes'

export const groupCharsIntoStrings = <N extends Node>(nodes: Array<NodeChar | N>) => { // group chars into strings
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

export function traverseTree(root: Node, cb: (node: Node) => void) {
  cb(root)

  for (const child of root.children) {
    traverseTree(child, cb)
  }
}

export function selectLinks(root: Node): NodeLink[] {
  const links: NodeLink[] = []

  traverseTree(root, (node) => {
    if (node instanceof NodeLink) {
      links.push(node)
    }
  })

  return links
}

export const createLink = (url: string, text: string = '') => text ? `[[${url}][${text}]]` : `[[${url}]]`
