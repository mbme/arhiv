import * as React from 'react'
import { ChronoFormatter } from '@v/chrono'
import {
  Box,
  useClickOnActivate,
  useFocusable,
} from '@v/web-platform'
import { Obj } from '@v/utils'
import { IDocument } from '../../api'
import { DocumentDataDescription, pickTitle } from '../../data-description'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

interface IProps<T extends string, P extends Obj> {
  document: IDocument<T, P>
  dataDescription: DocumentDataDescription<P>
  onActivate(document: IDocument<T, P>): void
}

export function CatalogEntry<T extends string, P extends Obj>(props: IProps<T, P>) {
  const {
    document,
    dataDescription,
    onActivate,
  } = props

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
      {pickTitle(document, dataDescription)}

      <Box
        as="small"
        display="block"
      >
        {dateFormat.format(new Date(document.updatedAt))}
      </Box>
    </Box>
  )
}
