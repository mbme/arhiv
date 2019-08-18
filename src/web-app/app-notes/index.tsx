import * as React from 'react'
import { Redirect } from '~/web-router'

import { NotFound } from '../parts'
import { IApp } from '../chrome'

import { NotesListView } from './NotesListView'
import { NoteViewContainer } from './NoteView'
import { NoteEditorViewContainer } from './NoteEditorView'

export const NotesApp: IApp = {
  name: 'Notes',
  rootRoute: '/notes',
  routes: {
    '/': () => (
      <Redirect to={{ path: '/notes' }} />
    ),

    '/notes': ({ filter }) => (
      <NotesListView filter={filter || ''} />
    ),

    '/note': ({ id }) => {
      if (!id) {
        return NotFound
      }

      return (
        <NoteViewContainer id={id} />
      )
    },

    '/note-editor': ({ id }) => (
      <NoteEditorViewContainer id={id} />
    ),
  },
}
