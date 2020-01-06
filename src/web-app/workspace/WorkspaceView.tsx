import * as React from 'react'
import {
  Box,
} from '~/web-platform'
import { DeckOfCards } from './DeckOfCards'
import { OpenCards } from './OpenCards'

interface IProps {
  ids: string[],
  filter: string,
}

export function WorkspaceView({ ids, filter }: IProps) {
  return (
    <Box
      display="grid"
      gridTemplateColumns="auto 1fr"
      height="100%"
    >
      <DeckOfCards filter={filter} />

      <OpenCards ids={ids} />
    </Box>
  )
}
