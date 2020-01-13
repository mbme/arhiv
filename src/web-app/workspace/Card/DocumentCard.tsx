import * as React from 'react'
import {
  Box,
} from '~/web-platform'
import { prettyPrintJSON } from '~/utils'
import { Document } from '~/arhiv/replica'
import { Frame } from './Frame'
import { Metadata } from './Metadata'
import { CloseIcon } from './CloseIcon'

interface IProps {
  document: Document
}

export function DocumentCard({ document }: IProps) {
  return (
    <Frame
      tabs={['document', 'metadata']}
      buttons={<CloseIcon documentId={document.id} />}
    >
      {(activeTabId) => {
        if (activeTabId === 'document') {
          return (
            <Box
              fontFamily="mono"
              wordBreak="break-word"
            >
              {prettyPrintJSON(document.props)}
            </Box>
          )
        }

        if (activeTabId === 'metadata') {
          return (
            <Metadata document={document} />
          )
        }

        return null
      }}
    </Frame>
  )
}
