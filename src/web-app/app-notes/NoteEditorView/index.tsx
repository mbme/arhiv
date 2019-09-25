import * as React from 'react'
import { noop } from '~/utils'
import { ReactiveValue } from '~/utils/reactive-value'
import { useReactiveValue } from '~/utils/react'
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

  // get or create the note
  const note = useReactiveValue(() => {
    if (id) {
      return arhiv.notes.getDocument$(id)
    }

    return new ReactiveValue<NoteDocument | undefined>(arhiv.notes.createNote())
  }, [id])

  // acquire note lock
  const [hasLock, setHasLock] = React.useState(false)
  React.useEffect(() => {
    if (!note) {
      return noop
    }

    const lock$ = note.acquireLock$()
    lock$.subscribe({
      next: setHasLock,
    })

    return lock$.complete
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
