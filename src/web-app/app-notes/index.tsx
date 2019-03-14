import * as React from 'react'
import {
  Redirect,
} from '~/web-router'

export default {
  name: 'Notes',
  rootRoute: '/notes',
  routes: {
    '/': () => <Redirect to={{ path: '/notes' }} />,
    '/notes': () => <h1>Notes view</h1>,
    '/note': () => <h1>Note view</h1>,
    '/note-editor': () => <h1>Note editor view</h1>,
  },
}
