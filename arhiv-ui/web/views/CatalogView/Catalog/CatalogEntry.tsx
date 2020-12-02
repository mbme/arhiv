import * as React from 'react'
import { ChronoFormatter } from '@v/chrono'
import {
  Box,
  useClickOnActivate,
  useFocusable,
} from '@v/web-platform'
import { IDocument } from '../../../api'
import { useDataDescription } from '../../../data-manager'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

interface IProps {
  document: IDocument
  onActivate(document: IDocument): void
}

export function CatalogEntry(props: IProps) {
  const {
    document,
    onActivate,
  } = props

  const {
    titleField,
  } = useDataDescription(document.data.type)

  const ref = React.useRef<HTMLDivElement>(null)
  const isFocused = useFocusable(ref)

  useClickOnActivate(ref)

  return (
    <Box
      mb="small"
      p="small"
      border={isFocused ? 'active' : 'invisible'}
      cursor="pointer"
      onClick={() => onActivate(document)}
      ref={ref}
    >
      {document.data[titleField]}

      <Box
        as="small"
        display="block"
      >
        {dateFormat.format(new Date(document.updatedAt))}
      </Box>
    </Box>
  )
}
