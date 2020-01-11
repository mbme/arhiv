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
      alignX="left"
      p="medium"
      height="100%"
      overflowY="scroll"
    >
      {ws.openIds.map(id => (
        <Box
          key={id}
          flex="0 0 auto"
          mb="large"
        >
          <Card id={id} />
        </Box>
      ))}
    </Column>
  )
}
