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

  const tabs = {
    [document.type]: () => <Note name={document.name} data={document.data} />,
    'metadata': () => <Metadata document={document} />,
  }

  return (
    <Frame
      tabs={tabs}
      buttons={buttons}
    />
  )
}
