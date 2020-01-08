import * as React from 'react'
import {
  Box,
} from '~/web-platform'
import { DeckOfCards } from './DeckOfCards'
import { OpenCards } from './OpenCards'

export function WorkspaceView() {
  return (
    <Box
      display="grid"
      gridTemplateColumns="auto 1fr"
      height="100%"
    >
      <DeckOfCards />

      <OpenCards />
    </Box>
  )
}
