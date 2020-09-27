import * as React from 'react'
import { ChronoFormatter } from '@v/chrono'
import {
  Box,
  useFocusable,
} from '@v/web-platform'
import { Note } from '../types'
import { RouterContext } from '@v/web-utils'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

interface IProps {
  note: Note
}

export function CatalogEntry({ note }: IProps) {
  const router = RouterContext.use()

  const ref = React.useRef<HTMLDivElement>(null)
  const isSelected = useFocusable(ref)

  return (
    <Box
      mb="small"
      p="small"
      border={isSelected ? '1px solid red' : '1px solid transparent'}
      cursor="pointer"
      onClick={() => router.push({ path: `/${note.id}` }) }
      innerRef={ref}
    >
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
