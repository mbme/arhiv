import * as React from 'react'
import { Card } from './Card'
import { Box } from '~/web-platform'
import { useWorkspaceManager } from './useWorkspaceManager'

export function OpenCards() {
  const ws = useWorkspaceManager()

  return (
    <Box
      display="flex"
      justifyContent="flex-start"
      alignItems="stretch"
      py="medium"
      overflowX="scroll"
    >
      {ws.openIds.map(id => (
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
