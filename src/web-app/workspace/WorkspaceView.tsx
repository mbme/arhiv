import * as React from 'react'
import {
  Column,
  Box,
} from '~/web-platform'
import { Catalog } from './Catalog'
import { OpenCards } from './OpenCards'
import { Header } from './Header'
import { useWorkspaceStore } from '../workspace-store'

export function WorkspaceView() {
  const store = useWorkspaceStore()

  return (
    <Column
      alignX="stretch"
      bgColor="bg2"
      height="100%"
    >
      <Header />

      <Box
        overflowY="scroll"
        flexGrow="1"
        hidden={!store.state.showCatalog}
        pt="medium"
      >
        <Catalog />
      </Box>

      <Box
        overflowY="scroll"
        flexGrow="1"
        hidden={store.state.showCatalog}
        pt="medium"
      >
        <OpenCards />
      </Box>
    </Column>
  )
}
