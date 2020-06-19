import * as React from 'react'
import {
  Row,
} from '@v/web-platform'
import { Tab } from './Tab'

interface IProps {
  activeTabId: string
  tabs: string[]
  onClick(tabId: string): void
}

export function TabBar({ tabs, activeTabId, onClick }: IProps) {
  return (
    <Row alignX="left">
      {tabs.map(tabId => (
        <Tab
          key={tabId}
          id={tabId}
          isActive={tabId === activeTabId }
          onClick={onClick}
        />
      ))}
    </Row>
  )
}
