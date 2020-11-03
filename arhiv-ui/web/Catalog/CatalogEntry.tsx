import * as React from 'react'
import { ChronoFormatter } from '@v/chrono'
import {
  Box,
  useClickOnActivate,
  useFocusable,
} from '@v/web-platform'
import { RouterContext } from '@v/web-utils'
import { Note } from '../api'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

interface IProps {
  note: Note
}

export function CatalogEntry({ note }: IProps) {
  const router = RouterContext.use()

  const ref = React.useRef<HTMLDivElement>(null)
  const isFocused = useFocusable(ref)

  useClickOnActivate(ref)

  return (
    <Box
      mb="small"
      p="small"
      border={isFocused ? 'active' : 'invisible'}
      cursor="pointer"
      onClick={() => router.push({ path: `/${note.id}` }) }
      ref={ref}
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
