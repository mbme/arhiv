import * as React from 'react'
import {
  Box,
  Row,
  theme,
} from '~/web-platform'
import { TabBar } from './TabBar'

interface ITabs {
  [key: string]: () => React.ReactNode
}

interface IProps {
  tabs: ITabs
  buttons: React.ReactNode
}

export function Frame({ tabs, buttons }: IProps) {
  const [activeTabId, setTabId] = React.useState(Object.keys(tabs)[0])

  return (
    <Box
      as="section"
    >
      <Row alignX="space-between">
        <TabBar
          activeTabId={activeTabId}
          tabs={Object.keys(tabs)}
          onClick={setTabId}
        />

        {buttons}
      </Row>

      <Box
        px="medium"
        pt="medium"
        border={theme.border}
        bgColor="bg0"
      >
        {tabs[activeTabId]()}
      </Box>
    </Box>
  )
}
