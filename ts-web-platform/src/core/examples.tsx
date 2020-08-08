import * as React from 'react'
import { Example } from '../Example'
import { Box } from '../Box'

const colors = [
  '--color-primary',
  '--color-secondary',
  '--color-text',
  '--color-textLight',
  '--color-heading',
  '--color-link',
  '--color-bg0',
  '--color-bg1',
  '--color-bg2',
  '--color-bgOverlay',
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
