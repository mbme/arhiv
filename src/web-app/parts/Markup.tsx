import React from 'react'
import { markupParser } from '~/markup-parser'
import { INode } from '~/parser-combinator'

interface IProps {
  value: string
}

function renderNode(node: INode<any>): React.ReactNode {
  switch (node.type) {
    case 'Markup': {
      const children = (node.value as Array<INode<any>>).map(renderNode)

      return React.createElement('article', {}, ...children)
    }

    case 'Paragraph': {
      const children = (node.value as Array<INode<any>>).map(renderNode)

      return React.createElement('p', {}, ...children)
    }

    case 'Header': {
      const [level, str] = node.value as [number, string]

      return React.createElement(`h${level}`, {}, str)
    }

    case 'UnorderedList': {
      const children = (node.value as Array<INode<any>>)
        .map(child => (
          <li>
            {renderNode(child)}
          </li>
        ))

      return React.createElement('ul', {}, ...children)
    }

    case 'CodeBlock': {
      const [lang, codeStr] = node.value as [string, string]

      return (
        <pre data-lang={lang}>
          {codeStr}
        </pre>
      )
    }

    case 'Link': {
      const [link, description] = node.value as [string, string]

      return (
        <a href={link}>
          {description}
        </a>
      )
    }

    case 'Mono':
      return (
        <code>
          {node.value as string}
        </code>
      )

    case 'Bold':
      return (
        <strong>
          {node.value as string}
        </strong>
      )

    case 'Striketrough':
      return (
        <s>
          {node.value as string}
        </s>
      )

    case 'Newlines':
      return null

    case 'String':
      return node.value as string

    default:
      throw new Error(`Unexpected node "${node.type}"`)
  }
}

export function Markup({ value }: IProps) {
  const result = markupParser.parseAll(value)

  if (!result.success) {
    return (
      <>
        <h1>Failed to parse markup:</h1>
        <pre>
          {JSON.stringify(result, null, 2)}
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
