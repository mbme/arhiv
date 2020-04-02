import * as React from 'react'
import { theme } from './theme'
import { Example } from '../Example'
import { Box } from '../Box'

export function StyleExamples() {
  return (
    <Example section title="Theme">
      <Example title="Colors">
        {Object.entries(theme.color).map(([name, value]) => (
          <Box
            key={name}
            height="3rem"
            backgroundColor={value}
            display="flex"
            alignItems="center"
            justifyContent="center"
          >
            {name}
          </Box>
        ))}
      </Example>
    </Example>
  )
}
