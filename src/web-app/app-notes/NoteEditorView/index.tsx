import * as React from 'react'
import { noop } from '~/utils'
import {
  useArhiv,
  NoteDocument,
} from '~/arhiv'
import { Heading } from '~/web-platform'
import { NotFound } from '~/web-app/parts'
import { NoteEditor } from './NoteEditor'

interface IProps {
  id?: string
}

export function NoteEditorViewContainer({ id }: IProps) {
  const arhiv = useArhiv()

  const [note, setNote] = React.useState<NoteDocument | undefined>(undefined)

  // get or create the note
  React.useEffect(() => {
    if (id) {
      return arhiv.notes.getDocument$(id).subscribe({ next: setNote })
    }

    setNote(arhiv.notes.create())

    return noop
  }, [])

  // acquire note lock
  const [hasLock, setHasLock] = React.useState(false)
  React.useEffect(() => {
    if (!note) {
      return noop
    }

    return note.acquireLock$().subscribe({ next: () => setHasLock(true) })
  }, [note])

  if (!note) {
    return NotFound
  }

  if (!hasLock) {
    return (
      <Heading>
        Note is in a read-only state, please wait
      </Heading>
    )
  }

  return (
    <NoteEditor note={note} />
  )
}
