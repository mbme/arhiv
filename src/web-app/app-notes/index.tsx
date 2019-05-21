import * as React from 'react'
import {
  Redirect,
  IParams,
} from '~/web-router'

import { NotFound } from '../parts'

import { NotesListView } from './NotesListView'
import { NoteView } from './NoteView'
import { NoteEditorView } from './NoteEditorView'

export default {
  name: 'Notes',
  rootRoute: '/notes',
  routes: {
    '/': () => <Redirect to={{ path: '/notes' }} />,

    '/notes': ({ filter }: IParams) => <NotesListView filter={filter || ''} />,

    '/note': ({ id }: IParams) => {
      if (!id) {
        return NotFound
      }

      return (
        <NoteView id={id} />
      )
    },

    '/note-editor': ({ id }: IParams) => <NoteEditorView key={id} id={id} />,
  },
}
