import * as React from 'react'
import {
  Heading,
  Button,
  Row,
} from '~/web-platform'
import { DocumentNote } from '~/arhiv/replica'
import {
  Markup,
} from '~/web-app/parts'
import { Frame } from './Frame'
import { Metadata } from './Metadata'
import { CloseIcon } from './CloseIcon'

interface IProps {
  document: DocumentNote
}

export function NoteCard({ document }: IProps) {
  const buttons = (
    <Row>
      <Button variant="link">
        Edit
      </Button>

      <CloseIcon documentId={document.id} />
    </Row>
  )

  return (
    <Frame
      tabs={[document.type, 'metadata']}
      buttons={buttons}
    >
      {(activeTabId) => {
        if (activeTabId === document.type) {
          return (
            <>
              <Heading
                letterSpacing="1.4px"
                fontSize="large"
              >
                {document.name}
              </Heading>

              <Markup value={document.data} />
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
