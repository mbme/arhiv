import * as React from 'react'
import { Card } from './Card'
import { Box } from '~/web-platform'

interface IProps {
  ids: string[],
}

export function OpenCards({ ids }: IProps) {
  return (
    <Box
      display="flex"
      justifyContent="flex-start"
      alignItems="stretch"
      py="medium"
      overflowX="scroll"
    >
      {ids.map(id => (
        <Box
          key={id}
          flex="0 0 auto"
          mx="medium"
        >
          <Card id={id} />
        </Box>
      ))}
    </Box>
  )
}
