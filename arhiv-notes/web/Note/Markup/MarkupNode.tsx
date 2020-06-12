import * as React from 'react'
import { nodes } from '../../markup-parser'
import { MarkupLink } from './MarkupLink'
import { StyleArg, useStyles } from '@v/web-platform'

const $article: StyleArg = {
  hyphens: 'auto',
  textAlign: 'justify',
}

interface IProps {
  node: nodes.Node
}

export function MarkupNode({ node }: IProps) {
  const articleClassName = useStyles($article)

  if (node instanceof nodes.NodeMarkup) {
    const children = node.children.map(child => <MarkupNode node={child} />)

    return React.createElement('article', { className: articleClassName }, ...children)
  }

  if (node instanceof nodes.NodeParagraph) {
    const children = node.children.map(child => <MarkupNode node={child} />)

    return React.createElement('p', {}, ...children)
  }

  if (node instanceof nodes.NodeHeader) {
    return React.createElement(`h${node.level}`, {}, node.value)
  }

  if (node instanceof nodes.NodeUnorderedList) {
    const children = node.children.map(child => (
      <li>
        <MarkupNode node={child} />
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
      <MarkupLink
        link={node.link}
        description={node.description}
      />
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
    // need to wrap into a Fragment due to https://github.com/DefinitelyTyped/DefinitelyTyped/issues/20544
    return (
      <>
        {node.value}
      </>
    )
  }

  throw new Error(`Unexpected node "${node.constructor.name}"`)
}
