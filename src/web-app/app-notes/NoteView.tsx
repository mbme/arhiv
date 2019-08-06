import * as React from 'react'
import {
  Icon,
  CleanLink,
} from '~/web-platform'
import {
  Toolbar,
  NotFound,
} from '../parts'
import { Note } from './Note'
import { useArhiv } from '~/arhiv';

interface IProps {
  id: string
}

export function NoteView({ id }: IProps) {
  const arhiv = useArhiv()
  const note = arhiv.notes.getNote(id)
  if (!note) {
    return NotFound
  }

  // FIXME isLocked
  const right = note.isLocked() || (
    <CleanLink to={{ path: '/note-editor', params: { id: note.id } }}>
      <Icon type="edit-2" />
    </CleanLink>
  )

  return (
    <>
      <Toolbar right={right} />

      <Note name={note.name} data={note.data} />
    </>
  )
}
