import * as React from 'react'
import {
  Box,
  Row,
  theme,
  Icon,
  stylish,
} from '~/web-platform'
import { Document } from '~/arhiv/replica'
import { useWorkspaceManager } from '../useWorkspaceManager'
import { Metadata } from './Metadata'

const $tab = stylish(
  {
    px: 'medium',
    py: '0.25rem',
    textAlign: 'center',
    textTransform: 'uppercase',
    fontSize: 'small',
    letterSpacing: '1.2px',
    background: theme.color.bgDarker,
    cursor: 'pointer',
    userSelect: 'none',
    minWidth: '7rem',
    '&:hover': {
      color: theme.color.link,
    },
  },
  props => props.active && {
    background: theme.color.bg,
    border: theme.border,
    borderBottom: '0 none',
    color: 'link',

    py: '0.3rem',
    // to hide part of container's border
    position: 'relative',
    top: '2px',
  },
)

interface IProps {
  document: Document
  children: React.ReactNode
}

export function CardFrame({ children, document }: IProps) {
  const ws = useWorkspaceManager()
  const [showMeta, setShowMeta] = React.useState(false)

  return (
    <Box
      as="section"
      width="37rem"
    >
      <Row alignX="left">
        <div
          className={$tab.with({ active: !showMeta }).className}
          onClick={() => setShowMeta(false)}
        >
          {document.type}
        </div>

        <Box width="2px" />

        <div
          className={$tab.with({ active: showMeta }).className}
          onClick={() => setShowMeta(true)}
        >
          Metadata
        </div>

        <Box flex="1" />

        <Icon
          type="x"
          title="close"
          onClick={() => ws.closeId(document.id)}
        />
      </Row>

      <Box
        px="medium"
        pt="medium"
        border={theme.border}
      >
        {showMeta ? <Metadata document={document} /> : children}
      </Box>
    </Box>
  )
}
