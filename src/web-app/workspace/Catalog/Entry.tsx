import * as React from 'react'
import { Document } from '~/arhiv/replica'
import { ChronoFormatter } from '~/chrono'
import {
  Box,
  theme,
  Label,
} from '~/web-platform'
import { useWorkspaceManager } from '../useWorkspaceManager'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

interface IProps {
  document: Document
}

export function CatalogEntry({ document }: IProps) {
  const ws = useWorkspaceManager()
  const isOpen = ws.openIds.includes(document.id)

  return (
    <Box
      onClick={() => ws.openId(document.id)}
      mb="small"
      p="small"
      cursor="pointer"
      bgColor={isOpen ? theme.color.bg0 : undefined}
    >
      <Label
        fontSize="fine"
      >
        {document.type}
      </Label>

      {document.getTitle()}

      <Box
        as="small"
        display="block"
      >
        {dateFormat.format(document.updatedAt)}
      </Box>
    </Box>
  )
}
