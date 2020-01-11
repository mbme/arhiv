import * as React from 'react'
import {
  Box,
} from '~/web-platform'
import { DeckOfCards } from './DeckOfCards'
import { OpenCards } from './OpenCards'
import { Chrome } from '../parts'

export function WorkspaceView() {
  return (
    <Chrome selected="workspace">
      <Box
        display="grid"
        gridTemplateColumns="auto 1fr"
        height="100%"
      >
        <DeckOfCards />

        <OpenCards />
      </Box>
    </Chrome>
  )
}
