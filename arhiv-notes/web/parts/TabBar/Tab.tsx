import * as React from 'react'
import {
  clickOnEnter,
  StyleArg,
  useStyles,
} from '@v/web-platform'

const $tab: StyleArg = {
  px: 'medium',
  py: '0.25rem',
  textAlign: 'center',
  textTransform: 'uppercase',
  fontSize: 'small',
  letterSpacing: '1.2px',
  background: 'bg1',
  cursor: 'pointer',
  userSelect: 'none',
  minWidth: '7rem',
  '&:hover': {
    color: 'var(--color-link)',
  },
}

const $tabActive: StyleArg = {
  background: 'bg0',
  border: 'default',
  borderBottom: '0 none',
  color: 'var(--color-link)',

  py: '0.3rem',
  // to hide part of container's border
  position: 'relative',
  top: '2px',
  mx: '2px',
  '&:first-child': {
    ml: '0',
  },
}

interface IProps {
  id: string
  isActive: boolean
  onClick(tabId: string): void
}

export function Tab({ id, isActive, onClick }: IProps) {
  const className = useStyles($tab, isActive && $tabActive)

  return (
    <div
      key={id}
      className={className}
      onClick={() => onClick(id)}
      onKeyPress={clickOnEnter}
      role="tab"
      tabIndex={0}
    >
      {id}
    </div>
  )
}
