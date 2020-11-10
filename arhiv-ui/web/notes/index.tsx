import * as React from 'react'
import { pathMatcher as pm } from '@v/utils'
import { Route } from '@v/web-utils'
import { Catalog } from './Catalog'
import { NoteCard } from './NoteCard'
import { NoteCardEditor } from './NoteCardEditor'

export const routes: Route<any>[] = [
  [pm`/notes`, () => <Catalog />],
  [pm`/notes/new`, () => <NoteCardEditor />],
  [pm`/notes/${'id'}`, ({ id }) => <NoteCard id={id} />],
  [pm`/notes/${'id'}/edit`, ({ id }) => <NoteCardEditor id={id} />],
]
