import * as React from 'react'
import { Button } from './Button'
import { Example } from './Example'

export function ButtonExamples() {
  return (
    <Example section title="Buttons">
      <Example title="Primary">
        <Button variant="primary">
          Primary Button
        </Button>
      </Example>

      <Example title="Primary disabled">
        <Button variant="primary" disabled>
          Primary Button
        </Button>
      </Example>

      <Example title="Secondary">
        <Button>
          Button
        </Button>
      </Example>

      <Example title="Secondary disabled">
        <Button disabled>
          Button
        </Button>
      </Example>

      <Example title="Link">
        <Button variant="link">
          Button
        </Button>
      </Example>

      <Example title="Link disabled">
        <Button variant="link" disabled>
          Button
        </Button>
      </Example>
    </Example>
  )
}
