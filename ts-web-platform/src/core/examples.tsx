import * as React from 'react'
import { theme } from './theme'
import { Examples } from '../Examples'
import { Box } from '../Box'

const examples = {
  'Colors': (
    <div>
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
    </div>
  ),
}

export function StyleExamples() {
  return (
    <Examples title="Theme" examples={examples} />
  )
}
