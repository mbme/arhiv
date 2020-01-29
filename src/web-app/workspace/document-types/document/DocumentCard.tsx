import * as React from 'react'
import { Document } from '~/arhiv/replica'
import { prettyPrintJSON } from '~/utils'
import { Box, Label, Row } from '~/web-platform'
import { CloseIcon, Frame, Metadata } from '../Card'

interface IProps {
  document: Document
}

export function DocumentCard({ document }: IProps) {
  const tabs = {
    [document.type]: () => (
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
    ),
    'metadata': () => <Metadata document={document} />,
  }

  return (
    <Frame
      tabs={tabs}
      buttons={<CloseIcon documentId={document.id} />}
    />
  )
}
