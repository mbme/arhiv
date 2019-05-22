import * as React from 'react'
import {
  useIsodb,
  Note,
} from '~/isodb-web-client'
import {
  NotFound,
} from '../../parts'
import { NoteEditor } from './NoteEditor'

interface IProps {
  id?: string
}

export function NoteEditorView({ id }: IProps) {
  const client = useIsodb()

  const [note, setNote] = React.useState<Note | undefined | null>(null)

  React.useEffect(() => {
    setNote(id ? client.notes.getNote(id) : client.notes.createNote())

    return () => {
      if (note && note.lock) {
        note.lock.release()
      }
    }
  }, [id])

  // acquire lock when possible
  React.useEffect(() => {
    if (note && !note.lock && !note.isLocked()) {
      note.acquireLock()
    }
  })

  if (note === null) {
    return null
  }

  if (note === undefined) {
    return NotFound
  }

  if (!note.lock) {
    return (
      <h1>Note is in a read-only state, please wait</h1>
    )
  }

  return (
    <NoteEditor note={note} />
  )
}
