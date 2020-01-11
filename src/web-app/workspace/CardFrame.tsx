import * as React from 'react'
import {
  Box,
  Row,
  theme,
  Icon,
} from '~/web-platform'
import { Document } from '~/arhiv/replica'
import { useWorkspaceManager } from './useWorkspaceManager'

interface IProps {
  document: Document
  children: React.ReactNode
}

export function CardFrame({ children, document }: IProps) {
  const ws = useWorkspaceManager()

  return (
    <Box
      as="article"
      width="37rem"
      border={theme.border}
    >
      <Row
        alignX="space-between"
        px="medium"
        pt="small"
      >
        <Box
          uppercase
          color="secondary"
          letterSpacing="1.2px"
          fontWeight="500"
          fontSize="small"
        >
          {document.type}
        </Box>

        <Box
          position="relative"
          left="6px"
        >
          <Icon
            type="x"
            title="close"
            onClick={() => ws.closeId(document.id)}
          />
        </Box>
      </Row>

      <Box
        px="medium"
        pt="medium"
      >
        {children}
      </Box>
    </Box>
  )
}
