import * as React from 'react'
import { useIsodb } from '~/isodb-web-client'
import {
  NotFound,
} from '../../parts'
import { NoteEditor } from './NoteEditor'

interface IProps {
  id?: string
}

export function NoteEditorView({ id }: IProps) {
  const client = useIsodb()

  let note = id ? client.notes.getNote(id) : null
  React.useEffect(() => { // create note if needed
    if (!id) {
      note = client.notes.createNote()
    }
  }, [id])

  if (!note) {
    return NotFound
  }

  if (note.isLocked()) {
    return (
      <h1>Note is being edited, please wait</h1>
    )
  }

  return (
    <NoteEditor key={note.id} note={note} />
  )
}
