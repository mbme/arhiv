import * as React from 'react'
import { Procedure } from '~/utils'
import {
  Button,
  Row,
} from '~/web-platform'
import { Note } from './Note'
import { CloseIcon, Metadata, Frame } from '../../Card'
import { DocumentNote } from '../types'

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
    [document.type]: () => <Note name={document.props.name} data={document.props.data} />,
    'metadata': () => <Metadata document={document} />,
  }

  return (
    <Frame
      tabs={tabs}
      buttons={buttons}
    />
  )
}
