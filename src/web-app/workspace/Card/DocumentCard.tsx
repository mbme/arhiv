import * as React from 'react'
import {
  Box,
  Label,
  Row,
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
      tabs={[document.type, 'metadata']}
      buttons={<CloseIcon documentId={document.id} />}
    >
      {(activeTabId) => {
        if (activeTabId === document.type) {
          return (
            <>
              <Row alignX="right">
                <Label color="danger">
                  deleted
                </Label>
              </Row>

              <Box
                fontFamily="mono"
                wordBreak="break-word"
                pb="medium"
                whiteSpace="pre-wrap"
              >
                {prettyPrintJSON(document.props)}
              </Box>
            </>
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
