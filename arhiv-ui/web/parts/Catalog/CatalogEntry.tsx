import * as React from 'react'
import { ChronoFormatter } from '@v/chrono'
import {
  Box,
} from '@v/web-platform'
import { RouterContext } from '@v/web-utils'
import { IDocument } from '../../api'
import { useDataDescription } from '../../data-manager'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

interface IProps {
  document: IDocument
}

export function CatalogEntry({ document }: IProps) {
  const router = RouterContext.use()

  const {
    titleField,
  } = useDataDescription(document.data.type)

  const ref = React.useRef<HTMLDivElement>(null)

  return (
    <Box
      mb="small"
      p="small"
      cursor="pointer"
      onClick={() => router.push(`/documents/${document.id}`)}
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
