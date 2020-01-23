import * as React from 'react'
import {
  Column,
  Box,
} from '~/web-platform'
import { Catalog } from './Catalog'
import { OpenCards } from './OpenCards'
import { useWorkspaceURLManager } from './useWorkspaceURLManager'
import { Header } from './Header'

export function WorkspaceView() {
  const ws = useWorkspaceURLManager()
  const [showCatalog, setShowCatalog] = React.useState(ws.openIds.length === 0)
  const [newestId, setNewestId] = React.useState('')

  return (
    <Column
      alignX="stretch"
      bgColor="bg2"
      height="100%"
    >
      <Header
        showCatalog={showCatalog}
        setShowCatalog={setShowCatalog}
        filter={ws.filter}
        updateFilter={filter => ws.updateFilter(filter)}
      />

      <Box
        overflowY="scroll"
        flexGrow="1"
        hidden={!showCatalog}
        pt="medium"
      >
        <Catalog
          filter={ws.filter}
          openIds={ws.openIds}
          openId={(id) => {
            ws.openId(id)
            setShowCatalog(false)
            setNewestId(id)
          }}
        />
      </Box>

      <Box
        overflowY="scroll"
        flexGrow="1"
        hidden={showCatalog}
        pt="medium"
      >
        <OpenCards
          openIds={ws.openIds}
          newestId={newestId}
        />
      </Box>
    </Column>
  )
}
