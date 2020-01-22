import * as React from 'react'
import {
  Box,
  Button,
  theme,
} from '~/web-platform'
import { Catalog } from './Catalog'
import { OpenCards } from './OpenCards'

export function WorkspaceView() {
  const [showCatalog, setShowCatalog] = React.useState(false)

  return (
    <Box
      minHeight="100%"
      pt="60px" // header
      bgColor="bg2"
    >
      <Box
        as="nav"
        boxShadow={theme.boxShadow}
        py="small"
        position="fixed"
        top="0"
        width="100%"
        bgColor="bg1"
        zIndex="1"
      >
        <Button
          variant="link"
          onClick={() => setShowCatalog(false)}
        >
          Workspace
        </Button>

        <Button
          variant="link"
          onClick={() => setShowCatalog(true)}
        >
          Catalog
        </Button>
      </Box>

      {showCatalog ? <Catalog /> : <OpenCards />}
    </Box>
  )
}
