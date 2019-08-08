import * as React from 'react'
import { Note as ArhivNote } from '~/arhiv'
import { useReactiveValue } from '~/utils/reactive'
import {
  Icon,
  CleanLink,
} from '~/web-platform'
import { Toolbar } from '../parts'
import { Note } from './Note'

interface IProps {
  note: ArhivNote
}

export function NoteView({ note }: IProps) {
  const locked = useReactiveValue(note.$locked)

  const right = locked || (
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
