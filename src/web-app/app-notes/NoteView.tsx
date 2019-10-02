import * as React from 'react'
import { useArhiv } from '~/arhiv'
import { useObservable } from '~/utils/react'
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
  id: string
}

export function NoteView({ id }: IProps) {
  const arhiv = useArhiv()
  const note = useObservable(() => arhiv.notes.getDocument$(id), [id])

  if (!note) {
    return NotFound
  }

  return (
    <>
      <Toolbar>
        <Spacer />

        <CleanLink to={{ path: '/note-editor', params: { id: note.id } }}>
          <Icon type="edit-2" />
        </CleanLink>
      </Toolbar>

      <Note
        name={note.record.name}
        data={note.record.data}
      />
    </>
  )
}
