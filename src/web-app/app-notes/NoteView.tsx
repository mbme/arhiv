import * as React from 'react'
import { useArhiv } from '~/arhiv'
import {
  Icon,
  CleanLink,
  Spacer,
  ProgressLocker,
  useObservable,
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
  const [note, isReady] = useObservable(() => arhiv.notes.getDocument$(id), [id])

  if (!isReady) {
    return (
      <ProgressLocker />
    )
  }

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
