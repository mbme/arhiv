import * as React from 'react'
import {
  stylish,
  Heading,
} from '~/web-platform'
import {
  markupParser,
  isFailure,
  nodes,
} from '~/markup-parser'

interface IProps {
  value: string
}

const $article = stylish({
  hyphens: 'auto',
  textAlign: 'justify',
})

function renderNode(node: nodes.Node): React.ReactNode {
  if (node instanceof nodes.NodeMarkup) {
    const children = node.children.map(renderNode)

    return React.createElement('article', { className: $article.className }, ...children)
  }

  if (node instanceof nodes.NodeParagraph) {
    const children = node.children.map(renderNode)

    return React.createElement('p', {}, ...children)
  }

  if (node instanceof nodes.NodeHeader) {
    return React.createElement(`h${node.level}`, {}, node.value)
  }

  if (node instanceof nodes.NodeUnorderedList) {
    const children = node.children.map(child => (
      <li>
        {renderNode(child)}
      </li>
    ))

    return React.createElement('ul', {}, ...children)
  }

  if (node instanceof nodes.NodeCodeBlock) {
    return (
      <pre data-lang={node.lang}>
        {node.value}
      </pre>
    )
  }

  if (node instanceof nodes.NodeLink) {
    return (
      <a href={node.link}>
        {node.description}
      </a>
    )
  }

  if (node instanceof nodes.NodeMono) {
    return (
      <code>
        {node.value}
      </code>
    )
  }

  if (node instanceof nodes.NodeBold) {
    return (
      <strong>
        {node.value}
      </strong>
    )
  }

  if (node instanceof nodes.NodeStrikethrough) {
    return (
      <s>
        {node.value}
      </s>
    )
  }

  if (node instanceof nodes.NodeNewlines) {
    return null
  }

  if (node instanceof nodes.NodeString) {
    return node.value
  }

  throw new Error(`Unexpected node "${node.constructor.name}"`)
}

export function Markup({ value }: IProps) {
  const result = markupParser.parseAll(value)

  if (isFailure(result)) {
    return (
      <>
        <Heading fontSize="medium">
          Failed to parse markup:
        </Heading>
        <pre>
          {result.toString()}
        </pre>
      </>
    )
  }

  return (
    <>
      {renderNode(result.result)}
    </>
  )
}
