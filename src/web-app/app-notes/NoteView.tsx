import * as React from 'react'
import {
  useArhiv,
  Note as ArhivNote,
} from '~/arhiv'
import { useReactiveValue } from '~/utils/reactive'
import {
  Icon,
  CleanLink,
  Spacer,
} from '~/web-platform'
import {
  Toolbar,
  NotFound,
} from '../parts'
import { Note } from './Note'

interface IProps {
  note: ArhivNote
}

function NoteView({ note }: IProps) {
  const locked = useReactiveValue(note.$locked)

  return (
    <>
      <Toolbar>
        <Spacer />

        {locked || (
          <CleanLink to={{ path: '/note-editor', params: { id: note.id } }}>
            <Icon type="edit-2" />
          </CleanLink>
        )}
      </Toolbar>

      <Note name={note.name} data={note.data} />
    </>
  )
}

export function NoteViewContainer({ id }: { id: string }) {
  const arhiv = useArhiv()
  const note = arhiv.notes.getNote(id)

  if (!note) {
    return NotFound
  }

  return (
    <NoteView note={note} />
  )
}
