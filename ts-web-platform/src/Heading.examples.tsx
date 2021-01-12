import * as React from 'react'
import { Example } from './Example'
import { Heading } from './Heading'

export function HeadingExamples() {
  return (
    <Example section title="Heading">
      <Heading variant="1">
        The longest title in the world
      </Heading>

      <Heading variant="2">
        The longest title in the world
      </Heading>

      <Heading variant="3">
        The longest title in the world
      </Heading>
    </Example>
  )
}
