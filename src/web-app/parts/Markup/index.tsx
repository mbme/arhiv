import * as React from 'react'
import {
  Heading,
} from '~/web-platform'
import {
  markupParser,
  isFailure,
} from '~/arhiv/markup-parser'
import { MarkupNode } from './MarkupNode'

interface IProps {
  value: string
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
    <MarkupNode node={result.value} />
  )
}
