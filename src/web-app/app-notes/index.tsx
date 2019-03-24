import * as React from 'react'
import {
  Redirect,
  IParams,
} from '~/web-router'
import { NotesListView } from './NotesListView'

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
        <h1>Note {id}</h1>
      )
    },

    '/note-editor': () => <h1>Note editor view</h1>,
  },
}
