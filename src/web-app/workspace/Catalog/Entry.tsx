import * as React from 'react'
import { Document } from '~/arhiv/replica'
import { ChronoFormatter } from '~/chrono'
import { Procedure } from '~/utils'
import { Box, Label, theme } from '~/web-platform'
import { getTitle } from '../document-types'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

interface IProps {
  document: Document
  isOpen: boolean
  onClick: Procedure
}

export function CatalogEntry({ document, isOpen, onClick }: IProps) {
  return (
    <Box
      onClick={onClick}
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

      {getTitle(document)}

      <Box
        as="small"
        display="block"
      >
        {dateFormat.format(document.updatedAt)}
      </Box>
    </Box>
  )
}
