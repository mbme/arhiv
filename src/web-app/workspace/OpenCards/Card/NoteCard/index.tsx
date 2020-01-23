import * as React from 'react'
import { DocumentNote } from '~/arhiv/replica'
import { NoteCardViewer } from './NoteCardViewer'
import { NoteCardEditor } from './NoteCardEditor'

interface IProps {
  document: DocumentNote
}

export function NoteCard({ document }: IProps) {
  const [editMode, setEditMode] = React.useState(false)

  if (editMode) {
    return (
      <NoteCardEditor
        document={document}
        onDone={() => setEditMode(false)}
      />
    )
  }

  return (
    <NoteCardViewer
      document={document}
      onEdit={() => setEditMode(true)}
    />
  )
}
