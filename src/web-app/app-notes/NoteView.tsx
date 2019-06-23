import * as React from 'react'
import { useIsodb } from '~/isodb-web-client'
import {
  Icon,
  CleanLink,
} from '~/web-platform'
import {
  Toolbar,
  NotFound,
} from '../parts'
import { Note } from './Note'

interface IProps {
  id: string
}

export function NoteView({ id }: IProps) {
  const client = useIsodb()
  const note = client.notes.getNote(id)
  if (!note) {
    return NotFound
  }

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
