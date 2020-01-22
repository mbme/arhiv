import * as React from 'react'
import {
  Row,
  theme,
  stylish,
} from '~/web-platform'

const $tab = stylish(
  {
    px: 'medium',
    py: '0.25rem',
    textAlign: 'center',
    textTransform: 'uppercase',
    fontSize: 'small',
    letterSpacing: '1.2px',
    background: theme.color.bg1,
    cursor: 'pointer',
    userSelect: 'none',
    minWidth: '7rem',
    '&:hover': {
      color: theme.color.link,
    },
  },
  props => props.active && {
    background: theme.color.bg0,
    border: theme.border,
    borderBottom: '0 none',
    color: 'link',

    py: '0.3rem',
    // to hide part of container's border
    position: 'relative',
    top: '2px',
    mx: '2px',
    '&:first-child': {
      ml: '0',
    },
  },
)

interface IProps {
  activeTabId: string
  tabs: string[]
  onClick(tabId: string): void
}

export function TabBar({ tabs, activeTabId, onClick }: IProps) {
  return (
    <Row alignX="left">
      {tabs.map(tabId => (
        <div
          key={tabId}
          className={$tab.with({ active: tabId === activeTabId }).className}
          onClick={() => onClick(tabId)}
          onKeyPress={(e) => {
            if (e.key === 'Enter') {
              onClick(tabId)
            }
          }}
          role="tab"
          tabIndex={0}
        >
          {tabId}
        </div>
      ))}
    </Row>
  )
}
