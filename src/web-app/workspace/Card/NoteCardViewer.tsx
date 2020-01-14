import * as React from 'react'
import { Procedure } from '~/utils'
import {
  Button,
  Row,
} from '~/web-platform'
import { DocumentNote } from '~/arhiv/replica'
import { Frame } from './Frame'
import { Metadata } from './Metadata'
import { CloseIcon } from './CloseIcon'
import { Note } from './Note'

interface IProps {
  document: DocumentNote
  onEdit: Procedure
}

export function NoteCardViewer({ document, onEdit }: IProps) {
  const buttons = (
    <Row>
      <Button variant="link" onClick={onEdit}>
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
            <Note name={document.name} data={document.data} />
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
