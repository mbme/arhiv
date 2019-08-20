import * as React from 'react'
import { noop } from '~/utils'
import {
  useArhiv,
  Note as ArhivNote,
} from '~/arhiv'
import { Heading } from '~/web-platform'
import { NotFound } from '~/web-app/parts'
import { NoteEditor } from './NoteEditor'

interface IProps {
  id?: string
}

export function NoteEditorViewContainer({ id }: IProps) {
  const arhiv = useArhiv(id ? true : false)

  const [note, setNote] = React.useState<ArhivNote | undefined | null>(null)
  const [hasLock, setHasLock] = React.useState(false)

  // get or create the note
  React.useEffect(() => {
    if (!note) {
      setNote(id ? arhiv.notes.getNote(id) : arhiv.notes.createNote())
    }
  })

  // acquire note lock
  React.useEffect(() => {
    if (note) {
      return note.$lock().subscribe(setHasLock)
    }

    return noop
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
