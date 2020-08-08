import * as React from 'react'
import { Example } from '../Example'
import { Box } from '../Box'

const colors = [
  '--color-primary',
  '--color-secondary',
  '--color-text',
  '--color-text-light',
  '--color-heading',
  '--color-link',
  '--color-bg0',
  '--color-bg-overlay',
]

export function StyleExamples() {
  return (
    <Example section title="Theme">
      <Example title="Colors">
        {colors.map(value => (
          <Box
            key={value}
            height="3rem"
            backgroundColor={`var(${value})`}
            display="flex"
            alignItems="center"
            justifyContent="center"
          >
            {value}
          </Box>
        ))}
      </Example>
    </Example>
  )
}
