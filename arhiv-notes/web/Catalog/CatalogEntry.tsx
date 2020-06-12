import * as React from 'react'
import { ChronoFormatter } from '@v/chrono'
import {
  Box,
  Label,
} from '@v/web-platform'
import { Note } from '../notes'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

interface IProps {
  note: Note
}

export function CatalogEntry({ note }: IProps) {
  return (
    <Box
      mb="small"
      p="small"
    >
      <Label
        fontSize="fine"
      >
        {note.type}
      </Label>

      {note.data.name}

      <Box
        as="small"
        display="block"
      >
        {dateFormat.format(new Date(note.updatedAt))}
      </Box>
    </Box>
  )
}
