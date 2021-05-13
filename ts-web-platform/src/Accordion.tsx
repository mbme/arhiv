import * as React from 'react'
import { StyleArg, StylishElement } from './core'

const $summary: StyleArg = {
  cursor: 'pointer',
}

interface IProps {
  summary: React.ReactNode
  children: React.ReactNode

  defaultOpen?: boolean
}

export function Accordion({ summary, children, defaultOpen = false }: IProps) {
  return (
    <details open={defaultOpen}>
      <StylishElement
        as="summary"
        $style={$summary}
      >
        {summary}
      </StylishElement>

      {children}
    </details>
  )
}
