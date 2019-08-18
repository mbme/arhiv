import * as React from 'react'
import { useReactiveValue } from '~/utils/reactive'
import {
  useArhiv,
  Note as ArhivNote,
} from '~/arhiv'
import { Heading } from '~/web-platform'
import { NotFound } from '~/web-app/parts'
import { NoteEditor } from './NoteEditor'

interface IProps {
  note: ArhivNote
}

function NoteEditorView({ note }: IProps) {
  const hasLock = useReactiveValue(note.$lock())

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

export function NoteEditorViewContainer({ id }: { id?: string }) {
  const arhiv = useArhiv(false)

  const note = id ? arhiv.notes.getNote(id) : arhiv.notes.createNote()

  if (!note) {
    return NotFound
  }

  return (
    <NoteEditorView note={note} />
  )
}
