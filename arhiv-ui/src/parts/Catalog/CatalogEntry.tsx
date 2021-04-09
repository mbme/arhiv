import * as React from 'react'
import { ChronoFormatter } from '@v/chrono'
import {
  Box,
  Row,
  StyleArg,
} from '@v/web-platform'
import { RouterContext } from '@v/web-utils'
import { IDocument } from '../../api'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

const $container: StyleArg = {
  border: 'invisible',
  '&:hover': {
    bgColor: 'rgb(160 231 251 / 47%)',
  }
}

const $preview: StyleArg = {
  display: '-webkit-box',
  '-webkit-line-clamp': 4,
  '-webkit-box-orient': 'vertical',
  lineClamp: 4,
  overflow: 'hidden',

  maxHeight: '10rem', // for images inside preview

  mb: 'fine',
}

interface IProps {
  document: IDocument
  preview: string
  showModificationDate: boolean
  showDataFields: string[]
}

export function CatalogEntry({ document, preview, showModificationDate, showDataFields }: IProps) {
  const router = RouterContext.use()

  return (
    <Box
      mb="small"
      p="small"
      cursor="pointer"
      onClick={() => router.push(`/documents/${document.id}`)}
      tabIndex="0"
      $style={$container}
    >
      <Box
        dangerouslySetInnerHTML={{ __html: preview }}
        $style={$preview}
      />

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
