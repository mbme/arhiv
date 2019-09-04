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
import { useArhiv } from '~/arhiv'

interface IProps {
  value: string
}

const $article = stylish({
  hyphens: 'auto',
  textAlign: 'justify',
})

function Link({ link, description }: { link: string, description: string }) {
  const arhiv = useArhiv()

  const attachment = arhiv.attachments.getAttachment(link)
  if (!attachment) {
    return (
      <a href={link}>
        {description || link}
      </a>
    )
  }

  if (attachment.attachment._mimeType.startsWith('image/')) {
    return (
      <img
        src={attachment.url}
        alt={description || link}
      />
    )
  }

  return (
    <a href={attachment.url} target="_blank" rel="noopener">
      {description || link}
    </a>
  )
}

function NodeRenderer({ node }: { node: nodes.Node }) {
  if (node instanceof nodes.NodeMarkup) {
    const children = node.children.map(child => <NodeRenderer node={child} />)

    return React.createElement('article', { className: $article.className }, ...children)
  }

  if (node instanceof nodes.NodeParagraph) {
    const children = node.children.map(child => <NodeRenderer node={child} />)

    return React.createElement('p', {}, ...children)
  }

  if (node instanceof nodes.NodeHeader) {
    return React.createElement(`h${node.level}`, {}, node.value)
  }

  if (node instanceof nodes.NodeUnorderedList) {
    const children = node.children.map(child => (
      <li>
        <NodeRenderer node={child} />
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
      <Link
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
    <NodeRenderer node={result.result} />
  )
}
