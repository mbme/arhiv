import * as React from 'react'
import { pathMatcher as pm } from '@v/utils'
import { Route } from '@v/web-utils'
import { NoteCatalog } from './NoteCatalog'
import { NoteCard } from './NoteCard'
import { NoteCardEditor } from './NoteCardEditor'

export const routes: Route<any>[] = [
  [pm`/notes`, () => <NoteCatalog />],
  [pm`/notes/new`, () => <NoteCardEditor />],
  [pm`/notes/${'id'}`, ({ id }) => <NoteCard id={id} />],
  [pm`/notes/${'id'}/edit`, ({ id }) => <NoteCardEditor id={id} />],
]
