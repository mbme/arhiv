import * as React from 'react'
import {
  useArhiv,
  NoteDocument,
} from '~/arhiv'
import { useReactiveValue, useReactiveValueMemo } from '~/utils/react'
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
  note: NoteDocument
}

function NoteView({ note }: IProps) {
  const locked = useReactiveValueMemo(() => note.$isLocked(), [note])

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

      <Note
        name={note.record.name}
        data={note.record.data}
      />
    </>
  )
}

export function NoteViewContainer({ id }: { id: string }) {
  const arhiv = useArhiv()
  const note = useReactiveValue(arhiv.notes.getDocument(id))

  if (!note) {
    return NotFound
  }

  return (
    <NoteView note={note} />
  )
}
