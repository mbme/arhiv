import * as React from 'react'
import {
  Box,
  Column,
} from '~/web-platform'
import { Card } from './Card'
import { useWorkspaceManager } from './useWorkspaceManager'

export function OpenCards() {
  const ws = useWorkspaceManager()

  return (
    <Column
      alignX="center"
    >
      {ws.openIds.map(id => (
        <Box
          key={id}
          flex="0 0 auto"
          mb="large"
          width="35rem"
          maxWidth="100%"
        >
          <Card id={id} />
        </Box>
      ))}
    </Column>
  )
}
