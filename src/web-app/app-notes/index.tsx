import * as React from 'react'
import {
  Redirect,
  IParams,
} from '~/web-router'
import { NotesListView } from './NotesListView'
import { NoteView } from './NoteView'

export default {
  name: 'Notes',
  rootRoute: '/notes',
  routes: {
    '/': () => <Redirect to={{ path: '/notes' }} />,

    '/notes': ({ filter }: IParams) => <NotesListView filter={filter || ''} />,

    '/note': ({ id }: IParams) => {
      if (!id) {
        return null
      }

      return (
        <NoteView id={id} />
      )
    },

    '/note-editor': () => <h1>Note editor view</h1>,
  },
}
