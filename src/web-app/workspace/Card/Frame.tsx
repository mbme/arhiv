import * as React from 'react'
import {
  Box,
  Row,
  theme,
} from '~/web-platform'
import { TabBar } from './TabBar'

interface IProps {
  tabs: string[]
  buttons: React.ReactNode
  children(tabId: string): React.ReactNode
}

export function Frame({ children, tabs, buttons }: IProps) {
  const [activeTabId, setTabId] = React.useState(tabs[0])

  return (
    <Box
      as="section"
      width="35rem"
    >
      <Row alignX="space-between">
        <TabBar
          activeTabId={activeTabId}
          tabs={tabs}
          onClick={setTabId}
        />

        {buttons}
      </Row>

      <Box
        px="medium"
        pt="medium"
        border={theme.border}
      >
        {children(activeTabId)}
      </Box>
    </Box>
  )
}
