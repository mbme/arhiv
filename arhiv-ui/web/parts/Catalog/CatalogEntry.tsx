import * as React from 'react'
import { ChronoFormatter } from '@v/chrono'
import {
  Box,
  Row,
  StyleArg,
} from '@v/web-platform'
import { RouterContext } from '@v/web-utils'
import { IDocument } from '../../api'
import { useDataDescription } from '../../data-manager'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

const $style: StyleArg = {
  border: 'invisible',
  '&:hover': {
    bgColor: 'rgb(160 231 251 / 47%)',
  }
}

interface IProps {
  document: IDocument
  showModificationDate: boolean
  showDataFields: string[]
}

export function CatalogEntry({ document, showModificationDate, showDataFields }: IProps) {
  const router = RouterContext.use()

  const {
    titleField,
  } = useDataDescription(document.data.type)

  return (
    <Box
      mb="small"
      p="small"
      cursor="pointer"
      onClick={() => router.push(`/documents/${document.id}`)}
      tabIndex="0"
      $style={$style}
    >
      {document.data[titleField]}

      <Row alignX="left">
        {showModificationDate && (
          <Box
            as="small"
            display="block"
          >
            {dateFormat.format(new Date(document.updatedAt))}
          </Box>
        )}
        {showDataFields.map(field => (
          <Box
            key={field}
            as="small"
            display="block"
            mr="medium"
          >
            {field}: {document.data[field]}
          </Box>
        ))}
      </Row>
    </Box>
  )
}
