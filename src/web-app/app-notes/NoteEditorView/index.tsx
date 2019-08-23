import * as React from 'react'
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

  const [note, setNote] = React.useState<NoteDocument | undefined | null>(null)
  const [hasLock, setHasLock] = React.useState(false)

  // get or create the note
  React.useEffect(() => {
    if (id) {
      return arhiv.notes.getDocument(id).subscribe(setNote)
    }

    setNote(arhiv.notes.createNote())

    return undefined
  }, [id])

  // acquire note lock
  React.useEffect(() => {
    if (note) {
      return note.$lock().subscribe(setHasLock)
    }

    return undefined
  }, [note])

  // null means no data yet, while undefined signals than we can't find the note
  if (!note) {
    return note === undefined ? NotFound : null
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
