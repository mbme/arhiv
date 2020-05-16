import * as React from 'react'
import { ChronoFormatter } from '@v/chrono'
import { Box } from '@v/web-platform'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

interface IProps {
  note: Note,
}

export function CatalogEntry({ note }: IProps) {
  return (
    <Box
      mb="small"
      p="small"
      cursor="pointer"
    >
      <Label
        fontSize="fine"
      >
        {document.type}
      </Label>

      {note.data.name}

      <Box
        as="small"
        display="block"
      >
        {dateFormat.format(document.updatedAt)}
      </Box>
    </Box>
  )
}
