import * as React from 'react'
import { usePromise } from '@v/web-utils'
import { createArray } from '@v/utils'
import {
  Box,
  StyleArg,
} from '@v/web-platform'
import { API, MarkupInlineNode, MarkupNode } from '../api'
import { MarkupLink } from './MarkupLink'

const $article: StyleArg = {
  hyphens: 'auto',
  textAlign: 'justify',
}

interface IProps {
  value: string
}

export function Markup({ value }: IProps) {
  const [nodes] = usePromise(() => API.parse_markup(value), [value])

  if (!nodes) {
    return null
  }

  return React.createElement(
    Box,
    {
      as: 'article',
      $style: $article,
    },
    ...nodes.map(renderMarkupNode).flat(1),
  )
}

function renderMarkupNode(node: MarkupNode) {
  if ('Newlines' in node) {
    return createArray(node.Newlines, () => <br />)
  }

  if ('Header' in node) {
    return (
      <h1>
        {node.Header}
      </h1>
    )
  }

  if ('Line' in node) {
    return node.Line.map(renderMarkupInlineNode)
  }

  throw new Error(`Got unexpected MarkupNode: ${Object.keys(node).join(', ')}`)
}

function renderMarkupInlineNode(inlineNode: MarkupInlineNode) {
  if ('String' in inlineNode) {
    return (
      inlineNode.String
    )
  }

  if ('Link' in inlineNode) {
    const [link, description] = inlineNode.Link

    return (
      <MarkupLink
        link={link}
        description={description}
      />
    )
  }

  if ('Bold' in inlineNode) {
    return (
      <strong>
        {inlineNode.Bold}
      </strong>
    )
  }

  if ('Mono' in inlineNode) {
    return (
      <code>
        {inlineNode.Mono}
      </code>
    )
  }

  if ('Strikethrough' in inlineNode) {
    return (
      <s>
        {inlineNode.Strikethrough}
      </s>
    )
  }

  throw new Error(`Got unexpected MarkupInlineNode: ${Object.keys(inlineNode).join(', ')}`)
}
