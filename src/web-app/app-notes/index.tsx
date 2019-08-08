import * as React from 'react'
import {
  Redirect,
  IParams,
} from '~/web-router'
import { Arhiv } from '~/arhiv'

import { NotFound } from '../parts'

import { NotesListView } from './NotesListView'
import { NoteView } from './NoteView'
import { NoteEditorView } from './NoteEditorView'

export default {
  name: 'Notes',
  rootRoute: '/notes',
  routes: {
    '/': () => (
      <Redirect to={{ path: '/notes' }} />
    ),

    '/notes': ({ filter }: IParams) => (
      <NotesListView filter={filter || ''} />
    ),

    '/note': ({ id }: IParams, arhiv: Arhiv) => {
      if (!id) {
        return NotFound
      }

      // FIXME this wouldn't update
      const note = arhiv.notes.getNote(id)
      if (!note) {
        return NotFound
      }

      return (
        <NoteView note={note} />
      )
    },

    '/note-editor': ({ id }: IParams, arhiv: Arhiv) => {
      const note = id ? arhiv.notes.getNote(id) : arhiv.notes.createNote()

      if (!note) {
        return NotFound
      }

      return (
        <NoteEditorView key={id} note={note} />
      )
    },
  },
}
